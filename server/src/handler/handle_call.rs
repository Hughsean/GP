use crate::Client;

/// a 主动呼叫，b 被动接听
pub async fn handle_call(a: Client, b: Client) -> anyhow::Result<()> {
    // 音频流
    // let audio_a = a.conn.open_bi().await?;
    // let audio_b = b.conn.open_bi().await?;
    // // 视频流
    // let video_a = a.conn.accept_bi().await?;
    // let video_b = b.conn.accept_bi().await?;

    // 音频
    let ca = a.clone();
    let cb = a.clone();
    let t1 = tokio::spawn(async move {
        loop {
            let audio_a = ca._conn.open_bi().await.unwrap();
            let audio_b = cb._conn.open_bi().await.unwrap();

            
            // Ok::<(), anyhow::Error>(())
        }
    });
    let t2 = tokio::spawn(async move {});
    let _ = tokio::join!(t1, t2);
    Ok(())
}

async fn transfer(
    (mut send_a, mut recv_a): (quic::SendStream, quic::RecvStream),
    (mut send_b, mut recv_b): (quic::SendStream, quic::RecvStream),
    // size: usize,
) -> anyhow::Result<()> {
    // a向b传送
    let t1 = async move {
        let mut buf: Vec<u8> = vec![0; 1000];
        loop {
            if recv_a.read_exact(buf.as_mut_slice()).await.is_ok() {
                if send_b.write_all(buf.as_slice()).await.is_ok() {
                    continue;
                }
            };
            break;
        }
    };
    // b向a传送
    let t2 = async move {
        let mut buf: Vec<u8> = vec![0; 1000];
        loop {
            if recv_b.read_exact(buf.as_mut_slice()).await.is_ok() {
                if send_a.write_all(buf.as_slice()).await.is_ok() {
                    continue;
                }
            }
            break;
        }
    };

    let t1 = tokio::spawn(t1);
    let t2 = tokio::spawn(t2);
    let _ = tokio::join!(t1, t2);
    Ok(())
}

// async fn handle_stream(
//     (mut send, mut recv): (quic::SendStream, quic::RecvStream),
// ) -> anyhow::Result<()> {
//     let data = recv.read_to_end(usize::MAX).await?;

//     send.write_all(&data).await?;
//     send.finish().await?;
//     Ok(())
// }
