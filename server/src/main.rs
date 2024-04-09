use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};

use config::Config;

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

    let listen: SocketAddr = config.listen.parse()?;

    let (certs, key) = {
        let key = fs::read(&config.key_path)?;
        let cert = fs::read(&config.cert_path)?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);
        (vec![cert], key)
    };

    let server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let mut server_config = quic::ServerConfig::with_crypto(Arc::new(server_crypto));

    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quic::Endpoint::server(server_config, listen)?;

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
