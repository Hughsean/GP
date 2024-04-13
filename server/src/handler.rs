use std::{sync::Arc, time::Duration};

use anyhow::anyhow;

use log::{debug, info, warn};
use quic::{Endpoint, SendStream};

use crate::{Client, ClientMap};

pub async fn handle_connection(
    conn: quic::Connecting,
    map: ClientMap,
    data_endpoint: Arc<tokio::sync::Mutex<Endpoint>>,
) -> anyhow::Result<()> {
    // 首先建立连接
    let conn = conn.await?;
    let client_addr = conn.remote_address();
    info!("连接建立 remote_addr({})", client_addr);
    async {
        // 只使用一个双向流
        // 接收流
        let stream = conn.accept_bi().await;
        let stream = match stream {
            Err(quic::ConnectionError::ApplicationClosed { .. }) => {
                warn!("连接关闭 remote_addr({})", client_addr);
                return Ok(());
            }
            Err(e) => {
                return Err(anyhow!("{}", e.to_string()));
            }
            Ok(s) => s,
        };

        let (send, mut recv) = stream;

        // 读取第一个请求
        match recv.read_to_end(usize::MAX).await {
            Ok(data) => match serde_json::from_slice::<common::Message>(&data) {
                Ok(msg) => {
                    //处理请求
                    info!("请求: {}", msg);
                    if let Err(e) =
                        handle_req(msg, map.clone(), send, conn.clone(), data_endpoint.clone())
                            .await
                    {
                        warn!("{}", e.to_string())
                    }
                }
                Err(e) => return Err(e.into()),
            },
            Err(e) => return Err(e.into()),
        };
        Ok(())
    }
    .await?;

    Ok(())
}

#[allow(dead_code)]
async fn handle_req(
    msg: common::Message,
    map: ClientMap,
    mut send: SendStream,
    ctrl_conn: quic::Connection,
    data_endpoint: Arc<tokio::sync::Mutex<Endpoint>>,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        common::Message::Wait(name) => {
            let mut clients_lock = map.lock().await;
            let dataendp_lock = data_endpoint.lock().await;

            if clients_lock.contains_key(&name) {
                // 用户名重复
                debug!("用户名({})重复", name);

                let msg = common::Message::Result(common::Info::Err);
                send.write_all(&msg.to_vec_u8()).await.unwrap();
                send.finish().await?;
            } else {
                let msg = common::Message::Result(common::Info::Ok);
                send.write_all(&msg.to_vec_u8()).await.unwrap();
                send.finish().await?;

                // 音频连接
                let a_conn = dataendp_lock.accept().await.unwrap().await?;
                // 视频连接
                let v_conn = dataendp_lock.accept().await.unwrap().await?;
                info!("音视频连接建立");

                info!("name({}) 加入等待接听列表", name);

                let is_waiting = Arc::new(tokio::sync::Mutex::new(true));
                let is_wating_c = is_waiting.clone();
                let is_waiting = Some(is_waiting);

                let ctrl_conn_c = ctrl_conn.clone();

                // 与客户端保活
                tokio::spawn(async move {
                    let wait = common::Message::Result(common::Info::Wait);
                    let wake = common::Message::Result(common::Info::Wake);
                    loop {
                        let mut send = ctrl_conn_c.open_uni().await?;
                        if *is_wating_c.lock().await {
                            send.write_all(&wait.to_vec_u8()).await?;
                            send.finish().await?;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        } else {
                            send.write_all(&wake.to_vec_u8()).await?;
                            send.finish().await?;
                            break;
                        }
                    }
                    Ok::<(), anyhow::Error>(())
                });
                clients_lock.insert(
                    name,
                    Client {
                        ctrl_conn,
                        a_conn,
                        v_conn,
                        is_waiting,
                    },
                );
            }
        }
        common::Message::Call(name) => {
            let mut clients_lock = map.lock().await;
            let dataendp_lock = data_endpoint.lock().await;

            let msg;

            let contains = clients_lock.contains_key(&name);
            //查看是否存在被呼叫用户
            if !contains {
                msg = common::Message::Result(common::Info::Err);
            } else {
                msg = common::Message::Result(common::Info::Ok);
            }

            send.write_all(&msg.to_vec_u8()).await?;
            send.finish().await?;

            if !contains {
                warn!("呼叫的用户({})不存在", name);
                return Ok(());
            }
            // 音频连接
            let a_conn = dataendp_lock.accept().await.unwrap().await?;
            // 视频连接
            let v_conn = dataendp_lock.accept().await.unwrap().await?;

            let c_active = Client {
                ctrl_conn,
                a_conn,
                v_conn,
                is_waiting: None,
            };
            let c_passive = clients_lock.remove(&name).unwrap();
            // 停止等待
            *c_passive.is_waiting.clone().unwrap().lock().await = false;

            drop(clients_lock);
            drop(dataendp_lock);

            handle_call(c_active, c_passive).await?;
        }
        // 请求等待呼叫用户列表
        common::Message::QueryUsers => {
            let clients_lock = map.lock().await;
            let mut v = vec![];
            for e in clients_lock.keys() {
                v.push(e.clone())
            }

            let msg = common::Message::Result(common::Info::UserList(v));
            send.write_all(&msg.to_vec_u8()).await.unwrap();
        }
        _ => return Err(anyhow!("时序错误")),
    }
    Ok(())
}

pub async fn handle_call(active: Client, passive: Client) -> anyhow::Result<()> {
    // 唤醒被呼叫者
    let msg = common::Message::Result(common::Info::Wake);
    let mut wake_sent = passive.ctrl_conn.open_uni().await?;
    wake_sent.write_all(&msg.to_vec_u8()).await?;
    wake_sent.finish().await?;

    // 转发数据
    info!("转发音频数据");
    let t1 = tokio::spawn(transfer(active.a_conn, passive.a_conn));
    // todo
    let t2 = tokio::spawn(transfer(active.v_conn, passive.v_conn));

    let _ = tokio::join!(t1, t2);
    Ok(())
}

async fn transfer(a: quic::Connection, p: quic::Connection) -> anyhow::Result<()> {
    let a_c = a.clone();
    let p_c = p.clone();

    // a to p
    let fut1 = async move {
        loop {
            if let (Ok(mut a_r), Ok(mut p_s)) = (a_c.accept_uni().await, p_c.open_uni().await) {
                if let Ok(data) = a_r.read_to_end(usize::MAX).await {
                    if p_s.write_all(&data).await.is_err() || p_s.finish().await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            };
        }
    };

    // p to a
    let fut2 = async move {
        loop {
            if let (Ok(mut p_r), Ok(mut a_s)) = (p.accept_uni().await, a.open_uni().await) {
                if let Ok(data) = p_r.read_to_end(usize::MAX).await {
                    if a_s.write_all(&data).await.is_err() || a_s.finish().await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    };
    let t1 = tokio::spawn(fut1);
    let t2 = tokio::spawn(fut2);

    let _ = tokio::join!(t1, t2);
    info!("数据转发停止");
    Ok(())
}
