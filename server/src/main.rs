use config::Config;
use log::{info, warn};
use std::{collections::HashMap, sync::Arc};

// const FRAME_LEN: usize = 691200;
// const AUDIO_LEN: usize = 960;

// const FRAME_MSG_BYTE_SIZE: usize = 1382411;
// const AUDIO_MSG_BYTE_SIZE: usize = 3851;

#[derive(Debug, Clone)]
struct Client {
    /// 备用连接
    pub _conn: quic::Connection,
    /// 音频连接
    pub a_conn: quic::Connection,
    /// 视频连接
    pub v_conn: quic::Connection, 
}

type ClientMap = Arc<tokio::sync::Mutex<HashMap<String, Client>>>;

fn main() {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    let config = Config::new(None);
    if let Err(e) = run(config) {
        println!("Err: {}", e.to_string())
    }
}

#[tokio::main]
async fn run(_config: Config) -> anyhow::Result<()> {
    let map = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let endpoint = common::make_endpoint(common::EndpointType::Server("0.0.0.0:12345".parse()?))?;

    let data_endpoint = Arc::new(tokio::sync::Mutex::new(common::make_endpoint(
        common::EndpointType::Server("0.0.0.0:12346".parse()?),
    )?));

    info!("监听 {}", endpoint.local_addr()?);
    //
    while let Some(conn) = endpoint.accept().await {
        let fut = handler::handle_connection(conn, map.clone(), data_endpoint.clone());
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                warn!("连接失败: {reason}", reason = e.to_string())
            }
        });
    }

    Ok(())
}

mod config;
mod handler;
