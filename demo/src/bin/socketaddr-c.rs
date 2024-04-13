use std::{fs, net::SocketAddr, sync::Arc};

use anyhow::anyhow;

fn main() {
    if let Err(e) = run() {
        println!("{}", e.to_string())
    }
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let (certs, key) = {
        let key = fs::read("cert/key.der")?;
        let cert = fs::read("cert/cert.der")?;

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

    let listen: SocketAddr = "0.0.0.0:0".parse()?;
    let mut endpoint = quic::Endpoint::server(server_config, listen)?;

    // client
    {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(&rustls::Certificate(fs::read("cert/cert.der")?))?;

        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let client_config = quic::ClientConfig::new(Arc::new(client_crypto));
        endpoint.set_default_client_config(client_config);
    }

    let remote_addr: SocketAddr = "122.51.128.39:12345".parse()?;

    let conn1 = endpoint.connect(remote_addr, "localhost")?.await?;
    let _ = conn1.open_bi().await?;

    println!("waiting...");

    let conn2 = endpoint.accept().await.ok_or(anyhow!("none"))?.await?;
    println!("{}", conn2.remote_address().to_string());

    conn1.close(0u8.into(), b"done");
    // endpoint.wait_idle().await;

    Ok(())
}
