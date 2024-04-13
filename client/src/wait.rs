use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use common::Message;

use quic::{Connection, Endpoint};

//
pub async fn wait(
    ctrl_endp: Endpoint,
    data_endp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    let conn = ctrl_endp.connect(ctrl_addr, server_name)?;
    let conn = conn.await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    let msg = Message::Wait(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    // 发送第一个请求
    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    if let Message::Result(info) = result {
        if info.is_ok() {
            let a_conn = data_endp.connect(data_addr, server_name)?.await?;
            let v_conn=data_endp.connect(data_addr, server_name)?.await?;
            waitcall(conn.clone(),a_conn,v_conn).await?;
        } else {
            return Err(anyhow!("请求错误"));
        }
    }

    Ok(conn)
}

async fn waitcall(_conn: Connection, a_conn: Connection, v_conn: Connection) -> anyhow::Result<()> {
    // 音频
    let (mut audio_send, mut audio_recv) = _conn.accept_bi().await?;
    // 视频
    let (mut video_send, video_recv) = _conn.open_bi().await?;

    Ok(())
}
