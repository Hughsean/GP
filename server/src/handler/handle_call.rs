use std::{io::Read, thread::sleep, time::Duration};

use crate::{Client, AUDIO_MSG_BYTE_SIZE, FRAME_MSG_BYTE_SIZE};

/// a 主动呼叫，b 被动接听
pub async fn handle_call(a: Client, b: Client) -> anyhow::Result<()> {
    // 音频流
    let audio_a = a.conn.open_bi().await?;
    let audio_b = b.conn.open_bi().await?;
    // 视频流
    let video_a = a.conn.accept_bi().await?;
    let video_b = b.conn.accept_bi().await?;

    let t1 = tokio::spawn(async move {
        if !transfer(audio_a, audio_b, AUDIO_MSG_BYTE_SIZE)
            .await
            .is_ok()
        {
            println!("错误")
        }
    });
    let t2 = tokio::spawn(async move {
        if !transfer(video_a, video_b, FRAME_MSG_BYTE_SIZE)
            .await
            .is_ok()
        {
            println!("错误")
        }
    });
    let _ = tokio::join!(t1, t2);
    Ok(())
}

async fn transfer(
    (mut send_a, mut recv_a): (quic::SendStream, quic::RecvStream),
    (mut send_b, mut recv_b): (quic::SendStream, quic::RecvStream),
    size: usize,
) -> anyhow::Result<()> {
    let t1 = async move {
        let mut buf: Vec<u8> = vec![0; size];
        loop {
            if recv_a.read_exact(buf.as_mut_slice()).await.is_ok() {
                if send_b.write_all(buf.as_slice()).await.is_ok() {
                    continue;
                }
            };
            break;
        }
    };
    let t2 = async move {
        let mut buf: Vec<u8> = vec![0; FRAME_MSG_BYTE_SIZE];
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
