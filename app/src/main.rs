#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::SocketAddr, process::exit, sync::Arc, thread::sleep, time::Duration};

use base64::Engine;
use call::call;
use client::Client;
use common::{
    endpoint_config::{make_endpoint, EndpointType},
    message::{Message, Res},
};
use opencv::{
    core::{Mat, MatTraitConst, VectorToVec},
    imgproc::resize,
    videoio::{VideoCapture, VideoCaptureTrait},
};
use wait::wait;

struct App {
    pub client: std::sync::Mutex<Option<Client>>,
}

fn main() {
    let state = App {
        client: std::sync::Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            init, wait, call, query, close, test
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod call;
mod wait;

#[tauri::command]
/// 初始化
fn init(addr: &str, name: &str, state: tauri::State<App>) -> Result<(), String> {
    tauri::async_runtime::block_on(async {
        let ctrl_addr = String::from(addr) + ":12345";
        let data_addr = String::from(addr) + ":12346";

        let ctrl_addr: SocketAddr = ctrl_addr.parse()?;
        let data_addr: SocketAddr = data_addr.parse()?;
        let endp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse()?))?;

        let (mut s, mut r) = endp
            .connect(ctrl_addr, "localhost")?
            .await?
            .open_bi()
            .await?;

        s.write_all(&Message::Hello.to_vec_u8()).await?;
        s.finish().await?;

        let res = r.read_to_end(usize::MAX).await?;

        let msg = serde_json::from_slice::<Message>(&res)?;

        if let Message::Response(Res::Ok) = msg {
            if state.client.lock().unwrap().is_none() {
                let client = client::Client::new(ctrl_addr, data_addr, name.into())?;
                state.client.lock().unwrap().replace(client);
            }
            anyhow::Ok(())
        } else {
            Err(anyhow::anyhow!("响应错误"))
        }
    })
    .or(Err("连接测试错误"))?;
    Ok(())
}

#[tauri::command]
fn close() {
    exit(0);
}

#[tauri::command]
fn test(win: tauri::Window) {
    for _ in 0..10 {
        win.emit("test", ()).unwrap();
        sleep(Duration::from_secs(1));
    }
}

#[tauri::command]
fn query(addr: &str) -> Result<Vec<String>, String> {
    tauri::async_runtime::block_on(async {
        let ctrl_addr = String::from(addr) + ":12345";

        let ctrl_addr: SocketAddr = ctrl_addr.parse()?;
        let endp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse()?))?;

        let (mut s, mut r) = endp
            .connect(ctrl_addr, "localhost")?
            .await?
            .open_bi()
            .await?;

        s.write_all(&Message::QueryUsers.to_vec_u8()).await?;
        s.finish().await?;

        let res = r.read_to_end(usize::MAX).await?;

        let msg = serde_json::from_slice::<Message>(&res)?;

        if let Message::Response(Res::UserList(users)) = msg {
            println!("OK");
            Ok(users)
        } else {
            Err(anyhow::anyhow!("响应错误"))
        }
    })
    .or(Err("连接错误".into()))
}

fn display_c(
    recv: std::sync::mpsc::Receiver<Vec<u8>>,
    stop: Arc<std::sync::RwLock<bool>>,
    win: tauri::Window,
) -> anyhow::Result<()> {
    loop {
        if *stop.read().unwrap() {
            break;
        }
        match recv.recv() {
            Ok(data) => {
                let buf = opencv::types::VectorOfu8::from(data);

                let base64str = base64::engine::general_purpose::STANDARD.encode(buf.as_slice());

                win.emit("play_frame", base64str).unwrap();
            }
            Err(e) => return Err(anyhow::anyhow!("{e}")),
        }
    }
    Ok(())
}

fn capture_c(
    cam: Arc<std::sync::Mutex<VideoCapture>>,
    send: std::sync::mpsc::Sender<Vec<u8>>,
    stop: Arc<std::sync::RwLock<bool>>,
) -> anyhow::Result<()> {
    let mut frame = Mat::default();
    loop {
        if *stop.read().unwrap() {
            break;
        }
        cam.lock().unwrap().read(&mut frame)?;
        if frame.size()?.width > 0 {
            let mut new_frame = Mat::default();

            resize(
                &frame,
                &mut new_frame,
                opencv::core::Size::new(600, 400),
                0.0,
                0.0,
                opencv::imgproc::INTER_LINEAR,
            )?;

            let params = opencv::types::VectorOfi32::new();
            let mut buf = opencv::types::VectorOfu8::new();

            // 对图片编码
            opencv::imgcodecs::imencode(".jpg", &new_frame, &mut buf, &params)?;
            send.send(buf.to_vec())?;
            std::thread::sleep(Duration::from_millis(client::DELAY as u64));
        }
    }
    Ok(())
}
