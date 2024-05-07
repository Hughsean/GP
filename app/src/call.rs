use anyhow::anyhow;

use client::audio::{audio_uni, make_input_stream, make_output_stream};
use common::{
    endpoint_config::{make_endpoint, EndpointType},
    message::{Message, Res},
};
use cpal::traits::StreamTrait;
use opencv::videoio::VideoCapture;
use std::{net::SocketAddr, process::exit, sync::Arc};
use tauri::async_runtime::{self, Mutex};

use crate::App;

#[tauri::command]
/// 初始化
pub fn call(win: tauri::Window, state: tauri::State<App>) -> Result<(), String> {
    // let winc = win.clone();

    match state.client.lock().unwrap().take() {
        Some(c) => {
            std::thread::spawn(move || {
                async_runtime::block_on(async move {
                    let _ = call_inner(
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

async fn call_inner(
    // endp: Endpoint,
    // aendp: Endpoint,
    // vendp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
    cam: Arc<std::sync::Mutex<VideoCapture>>,
    stop: Arc<std::sync::RwLock<bool>>,
    win: tauri::Window,
) -> anyhow::Result<()> {
    let endp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse()?))?;

    //---------------------
    let (ainput_send, ainput_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (aoutput_send, aoutput_recv) = std::sync::mpsc::channel::<Vec<f32>>();

    let ainput_recv_a = Arc::new(Mutex::new(ainput_recv));
    let aoutput_send_a = Arc::new(Mutex::new(aoutput_send.clone()));

    let input_stream = make_input_stream(ainput_send.clone());
    let output_stream = make_output_stream(aoutput_recv);
    // info!("音频设备配置成功");
    //---------------------
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
    let msg = Message::Call(name.into());

    // 第一个请求
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;
    // debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;
    // debug!("读取请求结果");

    if let Message::Response(Res::Ok) = result {
        // 创建数据连接
        let a_conn = endp.connect(data_addr, server_name)?.await?;
        let v_conn = endp.connect(data_addr, server_name)?.await?;

        input_stream.play()?;
        output_stream.play()?;

        // info!("已建立音视频连接");
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
            ainput_recv_a.clone(),
            aoutput_send_a.clone(),
        ));

        let _ = async_runtime::spawn(client::video::video_chanel(
            v_conn.clone(),
            vinput_recv_a,
            voutput_send_a,
        ))
        .await;
        let _ = t3.await;
        input_stream.pause()?;
        output_stream.pause()?;
        *stop.write().unwrap() = true;

        exit(0);
        // let _ = t1.await;
        // let _ = t2.await;
    } else {
        return Err(anyhow!("请求错误"));
    }
}
