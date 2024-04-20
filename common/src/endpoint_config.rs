use std::{fs, net::SocketAddr, sync::Arc};

pub enum EndpointType {
    Server(SocketAddr),
    Client(SocketAddr),
}

pub fn make_endpoint(enable: EndpointType) -> anyhow::Result<quic::Endpoint> {
    let (cert, key) = {
        let key = fs::read("cert/key.der")?;
        let cert = fs::read("cert/cert.der")?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);
        (cert, key)
    };

    let certs = vec![cert.clone()];
    let mut endpoint;

    match enable {
        EndpointType::Server(listen) => {
            let server_crypto = rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)?;
            let mut server_config = quic::ServerConfig::with_crypto(Arc::new(server_crypto));
            let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
            transport_config.max_concurrent_uni_streams(u8::MAX.into());
            // transport_config.max_concurrent_uni_streams(u16::)
            // transport_config.keep_alive_interval(Some(Duration::from_millis(300)));
            // transport_config.max_idle_timeout(Some(IdleTimeout::try_from(Duration::from_secs(5))?));
            // transport_config.max_idle_timeout(None);
            endpoint = quic::Endpoint::server(server_config, listen)?;
        }
        EndpointType::Client(addr) => {
            let mut roots = rustls::RootCertStore::empty();
            roots.add(&cert)?;
            let client_crypto = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(roots)
                .with_no_client_auth();
            let client_config = quic::ClientConfig::new(Arc::new(client_crypto));

            endpoint = quic::Endpoint::client(addr)?;
            endpoint.set_default_client_config(client_config);
        }
    }
    // let client_config;
    Ok(endpoint)
}
