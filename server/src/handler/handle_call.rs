use log::info;

use crate::Client;

/// a 主动呼叫，p 被动接听
pub async fn handle_call(active: Client, passive: Client) -> anyhow::Result<()> {
    // 唤醒被呼叫者
    let msg = common::Message::Result(common::Info::Wake);
    passive
        .waker
        .unwrap()
        .lock()
        .await
        .write_all(&msg.to_vec_u8())
        .await?;
    // 转发数据
    let t1 = tokio::spawn(transfer(active.a_conn, passive.a_conn));
    let t2 = tokio::spawn(transfer(active.v_conn, passive.v_conn));

    let _ = tokio::join!(t1, t2);
    Ok(())
}

async fn transfer(a: quic::Connection, p: quic::Connection) -> anyhow::Result<()> {
    loop {
        let (mut a_s, mut a_r) = a.accept_bi().await?;
        let (mut p_s, mut p_r) = p.accept_bi().await?;

        // a to p
        let fut1 = async move {
            let data = a_r.read_to_end(usize::MAX).await?;
            p_s.write_all(&data).await?;
            p_s.finish().await?;
            Ok::<(), anyhow::Error>(())
        };
        // p to a
        let fut2 = async move {
            let data = p_r.read_to_end(usize::MAX).await?;
            a_s.write_all(&data).await?;
            a_s.finish().await?;
            Ok::<(), anyhow::Error>(())
        };
        let t1 = tokio::spawn(fut1);
        let t2 = tokio::spawn(fut2);

        let (r1, r2) = tokio::join!(t1, t2);
        if r1.unwrap().is_err() || r2.unwrap().is_err() {
            info!("数据转发停止");
            break;
        }
    }
    Ok(())
}
