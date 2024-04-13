use crate::{
    audio::{audio, make_input_stream, make_output_stream},
    video::{make_cam, video},
};
use anyhow::anyhow;
use cpal::{traits::StreamTrait, Stream};
use log::info;
use opencv::videoio::VideoCapture;
use quic::Endpoint;
use std::{net::SocketAddr, sync::Arc};

pub async fn call(
    ctrl_endp: Endpoint,
    data_endp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    // 音频处理
    let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let input_recv = Arc::new(tokio::sync::Mutex::new(input_recv));
    let output_send = Arc::new(tokio::sync::Mutex::new(output_send));

    // 音频
    let input_stream = make_input_stream(input_send);
    let output_stream = make_output_stream(output_recv);
    info!("音频设备配置成功");
    let cam = make_cam()?;
    info!("摄像头启动");

    let conn = ctrl_endp.connect(ctrl_addr, server_name)?;
    let conn = conn.await?;

    let (mut send, mut recv) = conn.open_bi().await?;

    let msg = common::Message::Call(name.into());

    // 第一个请求
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: common::Message = serde_json::from_slice(&result)?;

    if let common::Message::Result(common::Info::Ok) = result {
        // 创建数据连接
        let a_conn = data_endp.connect(data_addr, server_name)?.await?;
        let v_conn = data_endp.connect(data_addr, server_name)?.await?;

        calling(
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

    Ok(())
}

async fn calling(
    // conn: quic::Connection,
    a_conn: quic::Connection,
    v_conn: quic::Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
    cam: VideoCapture,
    input_stream: Stream,
    output_stream: Stream,
) -> anyhow::Result<()> {
    // 启动设备
    input_stream.play().unwrap();
    output_stream.play().unwrap();
    info!("音频设备启动");

    let t1 = tokio::spawn(audio(a_conn, input_recv, output_send));
    // 视频处理
    let t2 = tokio::spawn(video(v_conn, cam));
    // info!()
    let _ = tokio::join!(t1, t2);
    Ok(())
}

#[test]

fn f() {
    let buff32 = vec![12.1f32; 3];
    let mut bufu8: Vec<u8> = Vec::with_capacity(buff32.len() * 4);
    unsafe { bufu8.set_len(buff32.len() * 4) };
    bufu8.copy_from_slice(unsafe {
        core::slice::from_raw_parts(buff32.as_ptr() as *const u8, buff32.len() * 4)
    });

    let mut v: Vec<f32> = Vec::with_capacity(buff32.len());
    unsafe { v.set_len(buff32.len()) };
    v.copy_from_slice(unsafe {
        core::slice::from_raw_parts(bufu8.as_ptr() as *const f32, buff32.len())
    });

    println!("{:?}", v);
}
