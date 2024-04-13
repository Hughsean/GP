use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Context, Result};
use common::Message;

use cpal::{traits::StreamTrait, Stream};
use log::{debug, info};
use opencv::videoio::VideoCapture;
use quic::{Connection, Endpoint};

use crate::{
    audio::{audio, make_input_stream, make_output_stream},
    video::{make_cam, video},
};

//
pub async fn wait(
    ctrl_endp: Endpoint,
    data_endp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let input_recv = Arc::new(tokio::sync::Mutex::new(input_recv));
    let output_send = Arc::new(tokio::sync::Mutex::new(output_send));
    let input_stream = make_input_stream(input_send);
    let output_stream = make_output_stream(output_recv);
    info!("音频设备配置成功");
    let cam = make_cam()?;
    info!("摄像头启动");

    let ctrl_conn = ctrl_endp.connect(ctrl_addr, server_name)?;
    let ctrl_conn = ctrl_conn.await?;
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;

    let msg = Message::Wait(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    // 发送第一个请求
    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    if let Message::Result(info) = result {
        // 创建音视频连接
        let a_conn = data_endp.connect(data_addr, server_name)?.await?;
        let v_conn = data_endp.connect(data_addr, server_name)?.await?;
        info!("已创建音视频连接");
        if info.is_ok() {
            waitcall(
                ctrl_conn.clone(),
                a_conn,
                v_conn,
                input_recv,
                output_send,
                cam,
                input_stream,
                output_stream,
            )
            .await?;
        } else {
            return Err(anyhow!("请求错误"));
        }
    }

    Ok(ctrl_conn)
}

/// 被呼叫
async fn waitcall(
    ctrl_conn: Connection,
    a_conn: Connection,
    v_conn: Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
    cam: VideoCapture,
    input_stream: Stream,
    output_stream: Stream,
) -> anyhow::Result<()> {
    info!("等待被呼叫");
    loop {
        let mut wake_recv = ctrl_conn.accept_uni().await?;

        let data_recv = wake_recv.read_to_end(usize::MAX).await?;
        let msg: common::Message = serde_json::from_slice(&data_recv).context("信息解析错误")?;
        match msg {
            Message::Result(common::Info::Wait) => {
                debug!("收到服务器等待保活信息");
                continue;
            }
            Message::Result(common::Info::Wake) => break,
            _ => {
                return Err(anyhow!("错误信息"));
            }
        }
    }

    info!("被服务器唤醒");

    input_stream.play().unwrap();
    output_stream.play().unwrap();
    info!("音频设备启动");
    // 音频
    let t1 = tokio::spawn(audio(a_conn, input_recv, output_send));

    // 视频
    let t2 = tokio::spawn(video(v_conn, cam));
    let _ = tokio::join!(t1, t2);

    Ok(())
}
