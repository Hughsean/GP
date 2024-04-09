use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use common::Message;
use quic::{Connection, Endpoint, RecvStream, SendStream};

pub async fn wait(
    endpoint: Endpoint,
    remote_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    let conn = endpoint.connect(remote_addr, server_name)?;
    let conn = conn.await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    let msg = Message::Wait(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;

    let data: Message = serde_json::from_slice(&result)?;

    if let Message::Result(info) = data {
        if !info.is_ok() {
            return Err(anyhow!("请求错误"));
        }
    } else {
        println!("传输时序错误")
    }

    Ok(conn)
}

async fn waitcall(conn: Connection) -> anyhow::Result<()> {
    let (send, recv) = conn.accept_bi().await?;

    Ok(())
}
