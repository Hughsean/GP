use std::{net::SocketAddr, sync::Arc, thread::sleep, time::Duration};

use anyhow::{anyhow, Context, Result};

use common::Message;

use cpal::{traits::StreamTrait, Stream};
use log::{debug, error, info};
use opencv::videoio::VideoCapture;
use quic::{Connection, Endpoint};

use crate::{
    audio::{audio, make_input_stream, make_output_stream, vf32_to_vu8, vu8_to_vf32},
    video::{make_cam, video},
};

//
pub async fn wait(
    endp: Endpoint,
    aendp: Endpoint,
    vendp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    // let cam = make_cam()?;
    // info!("摄像头启动");
    //todo
    // let _window = opencv::highgui::named_window("Video", opencv::highgui::WINDOW_AUTOSIZE)?;

    let ctrl_conn = endp.connect(ctrl_addr, server_name)?.await?;
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;

    debug!("建立连接");

    // 发送第一个请求
    let msg = Message::Wait(name.into());
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;

    debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    debug!("获取请求结果");

    if let Message::Result(info) = result {
        // 创建音视频连接
        if info.is_ok() {
            debug!("请求被接受");
            let a_conn = aendp.connect(data_addr, server_name)?.await?;
            let v_conn = vendp.connect(data_addr, server_name)?.await?;

            info!("已创建音视频连接");

            let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
            let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

            let input_recv_a = Arc::new(tokio::sync::Mutex::new(input_recv));
            let output_send_a = Arc::new(tokio::sync::Mutex::new(output_send.clone()));

            let input_stream = make_input_stream(input_send.clone());
            let output_stream = make_output_stream(output_recv);

            info!("音频设备配置成功");
            info!("等待被呼叫");
            loop {
                let (_, mut wake_recv) = ctrl_conn.accept_bi().await?;

                let data_recv = wake_recv.read_to_end(usize::MAX).await?;
                let msg: common::Message =
                    serde_json::from_slice(&data_recv).context("信息解析错误")?;
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

            let t1 = tokio::spawn(audio(
                a_conn.clone(),
                input_recv_a.clone(),
                output_send_a.clone(),
            ));
            let _ = tokio::join!(t1);
        } else {
            return Err(anyhow!("请求错误"));
        }
    }

    Ok(ctrl_conn)
}

// 被呼叫
// async fn fun(ctrl_conn: Connection, a_conn: Connection) -> anyhow::Result<()> {
//     let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
//     let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

//     let input_recv_a = Arc::new(tokio::sync::Mutex::new(input_recv));
//     let output_send_a = Arc::new(tokio::sync::Mutex::new(output_send.clone()));

//     let input_stream = make_input_stream(input_send.clone());
//     let output_stream = make_output_stream(output_recv);

//     info!("音频设备配置成功");
//     info!("等待被呼叫");
//     loop {
//         let (_, mut wake_recv) = ctrl_conn.accept_bi().await?;

//         let data_recv = wake_recv.read_to_end(usize::MAX).await?;
//         let msg: common::Message = serde_json::from_slice(&data_recv).context("信息解析错误")?;
//         match msg {
//             Message::Result(common::Info::Wait) => {
//                 debug!("收到服务器等待保活信息");
//                 continue;
//             }
//             Message::Result(common::Info::Wake) => break,
//             _ => {
//                 return Err(anyhow!("错误信息"));
//             }
//         }
//     }

//     info!("被服务器唤醒");
//     input_stream.play().unwrap();
//     output_stream.play().unwrap();
//     info!("音频设备启动");

//     let t1 = tokio::spawn(audio(
//         a_conn.clone(),
//         input_recv_a.clone(),
//         output_send_a.clone(),
//     ));
//     let _ = tokio::join!(t1);

//     Ok(())
// }
