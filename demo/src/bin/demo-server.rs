// use apps::common;
use std::{fs, net::SocketAddr, sync::Arc};

fn main() {
    let t = run();
    println!("{:?}", t)
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let listen: SocketAddr = "127.0.0.1:12345".parse()?;

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
        println!("connection incoming");
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                println!("connection failed: {reason}", reason = e.to_string())
            }
        });
    }

    Ok(())
}

async fn handle_request(
    (mut send, mut recv): (quic::SendStream, quic::RecvStream),
) -> anyhow::Result<()> {
    let req = recv
        .read_to_end(usize::MAX)
        .await
        .map_err(|e| anyhow::anyhow!("failed reading request: {}", e))?;

    let r: demo::R = serde_json::from_slice(&req)?;
    println!("{r:?}");
    //
    let response = "from server";

    // Write the response
    send.write_all(response.as_bytes())
        .await
        .map_err(|e| anyhow::anyhow!("failed to send response: {}", e))?;

    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| anyhow::anyhow!("failed to shutdown stream: {}", e))?;
    Ok(())
}

async fn handle_connection(conn: quic::Connecting) -> anyhow::Result<()> {
    let connection = conn.await?;
    async {
        println!("established");
        // Each stream initiated by the client constitutes a new request.
        loop {
            let stream = connection.accept_bi().await;
            let stream = match stream {
                Err(quic::ConnectionError::ApplicationClosed { .. }) => {
                    println!("连接关闭");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };
            let fut = handle_request(stream);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    println!("failed: {reason}", reason = e.to_string());
                }else {
                    println!("流处理结束")
                }
            });
        }
    }
    .await?;
    Ok(())
}
