use std::net::SocketAddr;

use anyhow::anyhow;
use quic::Endpoint;

async fn call(
    endpoint: Endpoint,
    remote_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    let conn = endpoint.connect(remote_addr, server_name)?;
    let conn = conn.await?;

    let (mut send, mut recv) = conn.open_bi().await?;

    let msg = common::Message::Call(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: common::Message = serde_json::from_slice(&result).unwrap();

    if let common::Message::Result(common::Info::Ok) = result {
        // 音频
        let (mut audio_send, mut audio_recv) = conn.accept_bi().await?;
        // 视频
        let (mut video_send, video_recv) = conn.open_bi().await?;
    } else {
        return Err(anyhow!("请求错误"));
    }

    Ok(())
}



