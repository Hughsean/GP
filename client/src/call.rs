use crate::{
    audio::{audio, make_input_stream, make_output_stream, vf32_to_vu8, vu8_to_vf32},
    video::{make_cam, video},
};
use anyhow::anyhow;
use cpal::{traits::StreamTrait, Stream};
use tracing::{debug, error, info};
use opencv::videoio::VideoCapture;
use quic::{Connection, Endpoint};
use std::{net::SocketAddr, sync::Arc};

pub async fn call(
    endp: Endpoint,
    aendp: Endpoint,
    vendp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    // 音频处理

    //todo
    // let _window = opencv::highgui::named_window("Video", opencv::highgui::WINDOW_AUTOSIZE)?;

    let ctrl_conn = endp.connect(ctrl_addr, server_name)?;
    let ctrl_conn = ctrl_conn.await?;
    debug!("建立 ctrl_conn");

    let (mut send, mut recv) = ctrl_conn.open_bi().await?;
    let msg = common::Message::Call(name.into());

    // 第一个请求
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;
    debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: common::Message = serde_json::from_slice(&result)?;
    debug!("读取请求结果");

    if let common::Message::Result(common::Info::Ok) = result {
        // 创建数据连接
        let a_conn = aendp.connect(data_addr, server_name)?.await?;
        let v_conn = vendp.connect(data_addr, server_name)?.await?;
        info!("已建立音视频连接");

        let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
        let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

        let input_recv_a = Arc::new(tokio::sync::Mutex::new(input_recv));
        let output_send_a = Arc::new(tokio::sync::Mutex::new(output_send.clone()));

        let input_stream = make_input_stream(input_send.clone());
        let output_stream = make_output_stream(output_recv);
        info!("音频设备配置成功");
        // 音频
        output_stream.play().unwrap();
        input_stream.play().unwrap();
        info!("音频设备启动");

        let t1 = tokio::spawn(audio(
            a_conn.clone(),
            input_recv_a.clone(),
            output_send_a.clone(),
        ));
        // 视频
        let _ = tokio::join!(t1);
        info!("呼叫结束");
    } else {
        return Err(anyhow!("请求错误"));
    }

    Ok(())
}

// async fn _fun(a_conn: Connection) -> anyhow::Result<()> {
//     // 启动设备

//     let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
//     let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

//     let input_recv_a = Arc::new(tokio::sync::Mutex::new(input_recv));
//     let output_send_a = Arc::new(tokio::sync::Mutex::new(output_send.clone()));

//     let input_stream = make_input_stream(input_send.clone());
//     let output_stream = make_output_stream(output_recv);
//     info!("音频设备配置成功");
//     // 音频
//     output_stream.play().unwrap();
//     input_stream.play().unwrap();
//     info!("音频设备启动");

//     let t1 = tokio::spawn(audio(
//         a_conn.clone(),
//         input_recv_a.clone(),
//         output_send_a.clone(),
//     ));
//     // 视频
//     let _ = tokio::join!(t1);
//     info!("呼叫结束");
//     Ok(())
// }

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
