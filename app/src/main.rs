// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::Ipv4Addr;

use client::Client;
use common::{
    endpoint_config::{make_endpoint, EndpointType},
    message::{Message, Res},
};
use quic::Endpoint;

// #[tauri::command]
// fn play(win: tauri::Window, state: tauri::State<App>) {
//     // todo
//     let mut frame = Mat::default();
//     state.cam.blocking_lock().read(&mut frame).unwrap();
//     if frame.size().unwrap().width > 0 {
//         let params = opencv::types::VectorOfi32::new();
//         let mut buf = opencv::types::VectorOfu8::new();

//         // 对图片编码
//         opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params).unwrap();

//         std::thread::sleep(Duration::from_millis(DELAY as u64));
//         let base64str = base64::engine::general_purpose::STANDARD.encode(buf.as_slice());
//         println!("{}", &base64str);
//         // Ok(base64str)
//         win.emit("play_frame", base64str).unwrap();
//     }
// }
#[tauri::command]
fn init(addr: &str, name: &str, state: tauri::State<App>) -> Result<(), String> {
    drop(state.client.lock().unwrap().take());

    tauri::async_runtime::block_on(async {        
        let addr = String::from(addr) + ":12345";

        let (mut s, mut r) = state
            .endp
            .connect(addr.parse()?, "localhost")?
            .await?
            .open_bi()
            .await?;

        s.write_all(&Message::Hello.to_vec_u8()).await?;
        s.finish().await?;

        let res = r.read_to_end(usize::MAX).await?;

        let msg = serde_json::from_slice::<Message>(&res)?;

        if let Message::Response(Res::Ok) = msg {
            anyhow::Ok(())
        } else {
            Err(anyhow::anyhow!("响应错误"))
        }
    })
    .or(Err("连接测试错误"))?;

    let client = client::Client::new().or(Err("()"))?;

    state.client.lock().unwrap().replace(client);
    state.name.lock().unwrap().replace(String::from(name));

    Ok(())
}

struct App {
    pub client: std::sync::Mutex<Option<Client>>,
    pub name: std::sync::Mutex<Option<String>>,
    pub endp: Endpoint,
}

fn main() {
    let state = App {
        client: std::sync::Mutex::new(None),
        name: std::sync::Mutex::new(None),
        endp: make_endpoint(EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap(),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![init,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod call;
mod wait;
