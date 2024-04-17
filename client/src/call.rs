use crate::{
    audio::{audio_uni, make_input_stream, make_output_stream},
    video::{self, make_cam},
};
use anyhow::anyhow;
use cpal::traits::StreamTrait;

use quic::Endpoint;
use std::{net::SocketAddr, sync::Arc};
use tracing::{debug, info};

pub async fn call(
    endp: Endpoint,
    aendp: Endpoint,
    vendp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    //---------------------
    let (ainput_send, ainput_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (aoutput_send, aoutput_recv) = std::sync::mpsc::channel::<Vec<f32>>();

    let ainput_recv_a = Arc::new(tokio::sync::Mutex::new(ainput_recv));
    let aoutput_send_a = Arc::new(tokio::sync::Mutex::new(aoutput_send.clone()));

    let input_stream = make_input_stream(ainput_send.clone());
    let output_stream = make_output_stream(aoutput_recv);
    info!("音频设备配置成功");
    //---------------------
    //////////////////////////////////////////////
    // 视频设备
    let mut cam = make_cam()?;
    info!("摄像头启动");
    let (vinput_send, vinput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let (voutput_send, voutput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let vinput_recv_a = Arc::new(tokio::sync::Mutex::new(vinput_recv));
    let voutput_send_a = Arc::new(tokio::sync::Mutex::new(voutput_send.clone()));
    /////////////////////////////////////////////

    let ctrl_conn = endp.connect(ctrl_addr, server_name)?;
    let ctrl_conn = ctrl_conn.await?;
    debug!("建立 ctrl_conn");

    // 建立请求连接
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;
    // 第一个请求
    let msg = common::Message::Call(name.into());
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;
    debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: common::Message = serde_json::from_slice(&result)?;
    debug!("读取请求结果");

    if let common::Message::Server(common::Response::Ok) = result {
        // 创建数据连接
        let a_conn = aendp.connect(data_addr, server_name)?.await?;
        let v_conn = vendp.connect(data_addr, server_name)?.await?;

        input_stream.play()?;
        output_stream.play()?;

        info!("已建立音视频连接");
        let t1 = std::thread::spawn(move || {
            let _ = video::capture_c(&mut cam, vinput_send.clone());
        });
        let t2 = std::thread::spawn(move || {
            let _ = video::display_c(voutput_recv);
        });

        let t3 = tokio::spawn(audio_uni(
            a_conn.clone(),
            ainput_recv_a.clone(),
            aoutput_send_a.clone(),
        ));

        let _ = tokio::spawn(crate::video::video_chanel(
            v_conn.clone(),
            vinput_recv_a,
            voutput_send_a,
        ))
        .await;
        let _ = t3.await;
        let _ = t1.join();
        let _ = t2.join();

        info!("呼叫结束");
    } else {
        return Err(anyhow!("请求错误"));
    }

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
