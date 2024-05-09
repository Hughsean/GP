use connection::connection;
use std::{fs, net::SocketAddr, sync::Arc};
use transmission::transmission;
#[tokio::main]
async fn main() {
    tokio::join!(connection(), transmission());
}

mod connection;
mod transmission;

pub fn make_endpoint(addr: SocketAddr) -> anyhow::Result<quic::Endpoint> {
    let (cert, key) = {
        let key = fs::read("cert/key.der")?;
        let cert = fs::read("cert/cert.der")?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);
        (cert, key)
    };

    let certs = vec![cert.clone()];

    let server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let mut server_config = quic::ServerConfig::with_crypto(Arc::new(server_crypto));
    // server_config.crypto.
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(u8::MAX.into());

    // transport_config.
    // transport_config.max_concurrent_uni_streams(u16::)
    // transport_config.keep_alive_interval(Some(Duration::from_millis(300)));
    // transport_config.max_idle_timeout(Some(IdleTimeout::try_from(Duration::from_secs(5))?));
    // transport_config.max_idle_timeout(None);
    let endpoint = quic::Endpoint::server(server_config, addr)?;

    // let client_config;
    Ok(endpoint)
}
