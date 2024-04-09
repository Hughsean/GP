use std::{fs, net::SocketAddr, sync::Arc, thread::sleep, time::Duration};

use common::Message;

fn main() {
    let _ = run();
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let remote_addr: SocketAddr = "127.0.0.1:12345".parse()?;
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&rustls::Certificate(fs::read("cert/cert.der")?))?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let client_config = quic::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quic::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    let v: Vec<u8> = vec![0; 921600];
    let m = Message::Video(v);
    let m = serde_json::to_string(&m).unwrap();

    let conn = endpoint.connect(remote_addr, "localhost")?.await?;

    let (mut send, mut recv) = conn.open_bi().await?;

    let req = demo::R::Login("client a".into());

    send.write_all(m.as_bytes()).await?;
    println!("send,sleep");
    sleep(Duration::from_secs(6));
    // send.finish().await?;

    // let recv_data = recv.read_to_end(usize::MAX).await?;

    // println!("{}", String::from_utf8_lossy(&recv_data));

    // conn.close(0u8.into(), b"done");

    // endpoint.wait_idle().await;

    Ok(())
}
