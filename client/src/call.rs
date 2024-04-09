use std::net::SocketAddr;

use quic::Endpoint;

async fn call(
    endpoint: Endpoint,
    remote_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    let conn = endpoint.connect(remote_addr, server_name)?;
    let conn = conn.await?;

    let (send, recv) = conn.open_bi().await?;

    

    Ok(())
}
