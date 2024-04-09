use std::{fs, net::SocketAddr, sync::Arc};

use quic::Endpoint;

pub fn make_endpoint(config: Config) -> anyhow::Result<Endpoint> {
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
    Ok(endpoint)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    pub listen: String,
    pub cert_path: String,
    pub key_path: String,
}

impl Config {
    pub fn new(config: Option<&str>) -> Self {
        let default = Default::default();
        match config {
            Some(path) => match fs::read_to_string(path) {
                Ok(str) => toml::from_str::<Config>(&str).unwrap_or(default),
                Err(_) => default,
            },
            None => default,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:12345".into(),
            cert_path: "cert/cert.der".into(),
            key_path: "cert/key.der".into(),
        }
    }
}
