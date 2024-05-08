// use apps::common;
use std::{fs, net::SocketAddr, sync::Arc};

fn main() {
    let t = run();
    println!("{:?}", t)
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let listen: SocketAddr = "0.0.0.0:12345".parse()?;

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

    let endpoint = quic::Endpoint::server(server_config, listen)?;

    while let Some(conn) = endpoint.accept().await {
        println!("连接到来");
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                println!("连接错误: {}", e.to_string())
            }
        });
    }

    Ok(())
}

async fn handle_connection(conn: quic::Connecting) -> anyhow::Result<()> {
    let conn = conn.await?;
    println!("{}", conn.remote_address().to_string());

    let mut endp;
    {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(&rustls::Certificate(fs::read("cert/cert.der")?))?;

        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let client_config = quic::ClientConfig::new(Arc::new(client_crypto));

        endp = quic::Endpoint::client("0.0.0.0:0".parse()?)?;
        endp.set_default_client_config(client_config);
    }

    match endp.connect(conn.remote_address(), "localhost")?.await {
        Ok(_) => (),
        Err(e) => println!("endp.conn {}", e.to_string()),
    }

    // sleep(Duration::from_secs(1000)).await;

    Ok(())
}
