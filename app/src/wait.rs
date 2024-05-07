use std::{net::SocketAddr, process::exit, sync::Arc};

use anyhow::Context;
use client::{
    audio::{audio_uni, make_input_stream, make_output_stream},
    video,
};
use common::{
    endpoint_config::{make_endpoint, EndpointType},
    message::{Message, Res},
};
use cpal::traits::StreamTrait;
use opencv::videoio::VideoCapture;
use quic::Connection;
use tauri::async_runtime::{self, Mutex};

use crate::App;

#[tauri::command]
/// 初始化
pub fn wait(win: tauri::Window, state: tauri::State<App>) -> Result<(), String> {
    // let winc = win.clone();

    match state.client.lock().unwrap().take() {
        Some(c) => {
            std::thread::spawn(move || {
                async_runtime::block_on(async move {
                    let _ = wait_inner(
                        c.ctrl_addr,
                        c.data_addr,
                        "localhost",
                        &c.name,
                        c.cam,
                        c.stop,
                        win,
                    )
                    .await;
                });
            });
            Ok(())
        }
        None => Err("请求错误".into()),
    }
}

pub async fn wait_inner(
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
    cam: Arc<std::sync::Mutex<VideoCapture>>,
    stop: Arc<std::sync::RwLock<bool>>,
    win: tauri::Window,
) -> anyhow::Result<Connection> {
    let endp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse()?))?;

    // 音频设备
    let (a_input_send, a_input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (a_output_send, a_output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

    let a_input_recv_a = Arc::new(Mutex::new(a_input_recv));
    let a_output_send_a = Arc::new(Mutex::new(a_output_send.clone()));

    let input_stream = make_input_stream(a_input_send.clone());
    let output_stream = make_output_stream(a_output_recv);
    // info!("音频设备配置成功");
    //////////////////////////////////////////////
    // 视频设备
    // let mut cam = make_cam()?;
    // info!("摄像头启动");
    let (vinput_send, vinput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let (voutput_send, voutput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let vinput_recv_a = Arc::new(Mutex::new(vinput_recv));
    let voutput_send_a = Arc::new(Mutex::new(voutput_send.clone()));
    /////////////////////////////////////////////

    let ctrl_conn = endp.connect(ctrl_addr, server_name)?.await?;
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;

    // debug!("建立连接");

    // 发送第一个请求
    let msg = Message::Wait(name.into());
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;

    // debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    // debug!("获取请求结果");

    if let Message::Response(info) = result {
        // 创建音视频连接
        if info.is_ok() {
            // debug!("请求被接受");
            // let a_conn = endp.connect(data_addr, server_name)?.await?;
            // let v_conn = endp.connect(data_addr, server_name)?.await?;

            loop {
                let (_, mut wake_recv) = ctrl_conn.accept_bi().await?;

                let data_recv = wake_recv.read_to_end(usize::MAX).await?;
                let msg: Message = serde_json::from_slice(&data_recv).context("信息解析错误")?;
                match msg {
                    Message::Response(Res::Wait) => {
                        // debug!("收到服务器等待保活信息");
                        continue;
                    }
                    Message::Response(Res::Wake) => break,
                    _ => {
                        return Err(anyhow::anyhow!("错误信息"));
                    }
                }
            }

            let a_conn = endp.connect(data_addr, server_name)?.await?;
            let v_conn = endp.connect(data_addr, server_name)?.await?;

            win.emit("wake", ()).unwrap();
            //
            input_stream.play().unwrap();
            output_stream.play().unwrap();
            //
            let stopc = stop.clone();
            let _t1 = std::thread::spawn(move || {
                let _ = crate::capture_c(cam, vinput_send.clone(), stopc);
            });
            let stopc = stop.clone();
            let _t2 = std::thread::spawn(move || {
                let _ = crate::display_c(voutput_recv, stopc, win);
            });

            let t3 = async_runtime::spawn(audio_uni(
                a_conn.clone(),
                a_input_recv_a.clone(),
                a_output_send_a.clone(),
            ));
            let _ = async_runtime::spawn(video::video_chanel(
                v_conn.clone(),
                vinput_recv_a,
                voutput_send_a,
            ))
            .await;

            let _ = t3.await;

            exit(0);
            // input_stream.pause()?;
            // output_stream.pause()?;
            // *stop.write().await = true;
        } else {
            return Err(anyhow::anyhow!("请求错误"));
        }
    }
    Ok(ctrl_conn)
}
