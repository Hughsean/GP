use config::Config;
use tracing::{info, warn};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

// const FRAME_LEN: usize = 691200;
// const AUDIO_LEN: usize = 960;

// const FRAME_MSG_BYTE_SIZE: usize = 1382411;
// const AUDIO_MSG_BYTE_SIZE: usize = 3851;

#[derive(Debug, Clone)]
struct Client {
    /// 控制连接, 用于交换状态
    pub ctrl_conn: quic::Connection,
    /// 音频连接
    pub a_conn: quic::Connection,
    /// 视频连接
    pub v_conn: quic::Connection,
    ///
    pub ctrl: Option<Arc<tokio::sync::Mutex<Option<quic::Connection>>>>,
}

type ClientMap = Arc<tokio::sync::Mutex<HashMap<String, Client>>>;

fn main() {
    // std::env::set_var("RUST_LOG", "server=DEBUG");
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("server=debug")
        // .with_max_level(Level::DEBUG)
        .init();

    let config = Config::new(None);
    if let Err(e) = run(config) {
        println!("Err: {}", e.to_string())
    }
}

#[tokio::main]
async fn run(_config: Config) -> anyhow::Result<()> {
    let clients = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    let ctrl_listen = "0.0.0.0:12345".parse::<SocketAddr>()?;
    let data_listen = "0.0.0.0:12346".parse::<SocketAddr>()?;

    let ctrl_endp = common::make_endpoint(common::EndpointType::Server(ctrl_listen))?;
    let data_endp = common::make_endpoint(common::EndpointType::Server(data_listen))?;

    info!("监听 {}", ctrl_endp.local_addr()?);
    //
    while let Some(conn) = ctrl_endp.accept().await {
        let fut = handler::handle_connection(conn, clients.clone(), data_endp.clone());
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
mod exchange;