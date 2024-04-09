use std::{collections::HashMap, sync::Arc};

use config::{make_endpoint, Config};

const FRAME_LEN: usize = 691200;
const AUDIO_LEN: usize = 960;


const FRAME_MSG_BYTE_SIZE: usize = 1382411;
const AUDIO_MSG_BYTE_SIZE: usize = 3851;

#[derive(Debug)]
struct Client {
    pub conn: quic::Connection,
}

type ClientMap = Arc<tokio::sync::Mutex<HashMap<String, Client>>>;

fn main() {
    let config = Config::new(None);
    if let Err(e) = run(config) {
        println!("Err: {}", e.to_string())
    }
}

#[tokio::main]
async fn run(config: Config) -> anyhow::Result<()> {
    let map = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    let endpoint = make_endpoint(config)?;
    //
    while let Some(conn) = endpoint.accept().await {
        println!("连接创建");
        let fut = handler::handle_connection(conn, map.clone());
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                println!("连接失败: {reason}", reason = e.to_string())
            }
        });
    }

    Ok(())
}

mod config;
mod handler;
