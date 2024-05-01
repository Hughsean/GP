// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::Engine;
use client::DELAY;
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};

use std::{sync::Arc, time::Duration};
use tauri::{async_runtime::Mutex, Manager};

#[derive(Clone, serde::Serialize)]
struct Payload {
    base64: String,
}

#[tauri::command]
fn init(name: &str, addr: &str) -> Result<(), String> {
    println!("{name} {addr}");
    std::thread::sleep(Duration::from_secs(1));
    // todo
    Ok(())
}

#[tauri::command]
fn play(win: tauri::Window, state: tauri::State<App>) {
    // todo
    let mut frame = Mat::default();
    state.cam.blocking_lock().read(&mut frame).unwrap();
    if frame.size().unwrap().width > 0 {
        let params = opencv::types::VectorOfi32::new();
        let mut buf = opencv::types::VectorOfu8::new();

        // 对图片编码
        opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params).unwrap();

        std::thread::sleep(Duration::from_millis(DELAY as u64));
        let base64str = base64::engine::general_purpose::STANDARD.encode(buf.as_slice());
        println!("{}", &base64str);
        // Ok(base64str)
        win.emit("play_frame", base64str).unwrap();
    }
}

struct App {
    pub cam: Arc<Mutex<VideoCapture>>,
}

fn main() {
    let cam = Arc::new(Mutex::new(
        videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(),
    ));

    let state = App { cam };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![init, play])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
