use common::message::{Message, Res};
use quic::{Connection, Endpoint, SendStream};
use tracing::{debug, info, warn};

use crate::{exchange::exchange_uni, Client, ClientMap};

pub async fn call(
    name: String,
    clients: ClientMap,
    mut send: SendStream,
    data_endp: Endpoint,
    ctrl_conn: Connection,
) -> anyhow::Result<()> {
    let mut clients_lock = clients.write().await;

    debug!("获取锁");

    let msg;
    let contains = clients_lock.contains_key(&name);
    //查看是否存在被呼叫用户
    if !contains {
        msg = Message::Response(Res::Err);
    } else {
        msg = Message::Response(Res::Ok);
    }

    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;
    debug!("回传状态");

    if !contains {
        warn!("呼叫的用户({})不存在", name);
        return Ok(());
    }

    // 主动端音视频连接建立
    // 音频连接
    let a_conn = data_endp.accept().await.unwrap().await?;
    // 视频连接
    let v_conn = data_endp.accept().await.unwrap().await?;

    debug!("接收音视频连接");

    let c_active = Client {
        ctrl_conn,
        a_conn: Some(a_conn),
        v_conn: Some(v_conn),
        ctrl: None,
    };

    let mut c_passive = clients_lock.remove(&name).unwrap();
    // 停止等待
    let _ = c_passive.ctrl.clone().unwrap().lock().await.take();
    debug!("保活线程停止");

    // 唤醒被呼叫者
    let msg = Message::Response(Res::Wake);
    let (mut wake_sent, _) = c_passive.ctrl_conn.open_bi().await?;
    wake_sent.write_all(&msg.to_vec_u8()).await?;
    wake_sent.finish().await?;

    // 被动端音视频连接建立
    // 音频连接
    let a_conn = data_endp.accept().await.unwrap().await?;
    // 视频连接
    let v_conn = data_endp.accept().await.unwrap().await?;

    c_passive.a_conn = Some(a_conn);
    c_passive.v_conn = Some(v_conn);

    drop(clients_lock);
    handle_call(c_active, c_passive).await
}

async fn handle_call(active: Client, passive: Client) -> anyhow::Result<()> {
    // // 唤醒被呼叫者
    // let msg = Message::Response(Res::Wake);
    // let (mut wake_sent, _) = passive.ctrl_conn.open_bi().await?;
    // wake_sent.write_all(&msg.to_vec_u8()).await?;
    // wake_sent.finish().await?;

    // 转发数据
    let t1 = tokio::spawn(exchange_uni(
        active.a_conn.unwrap(),
        passive.a_conn.unwrap(),
        "音频",
    ));
    let t2 = tokio::spawn(exchange_uni(
        active.v_conn.unwrap(),
        passive.v_conn.unwrap(),
        "视频",
    ));
    info!("转发音视频数据");
    let _ = tokio::join!(t1, t2);
    Ok(())
}
