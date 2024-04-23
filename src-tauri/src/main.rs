// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn fun(name: &str, n: usize) -> String {
    format!("Hello, {}! n:{}", name, n)
}

#[tauri::command]
fn init(name: &str, addr: &str) -> Result<(), String> {
    println!("{name} {addr}");
    std::thread::sleep(Duration::from_secs(5));
    // todo
    Ok(())
}



fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fun, init])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
