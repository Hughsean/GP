use std::sync::Arc;

use anyhow::anyhow;
use common::Message;
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
    conn: quic::Connection,
    data_endpoint: Arc<tokio::sync::Mutex<Endpoint>>,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        common::Message::Wait(name) => {
            let mut map_lock = map.lock().await;
            let mut endp_lock = data_endpoint.lock().await;

            if map_lock.contains_key(&name) {
                // 用户名重复
                debug!("用户名({})重复", name);

                let msg = serde_json::to_string(&common::Message::Result(common::Info::Err))?;
                send.write_all(msg.as_bytes()).await.unwrap();
                send.finish().await?;
            } else {
                info!("{} 加入等待接听列表", name);

                let msg = serde_json::to_string(&common::Message::Result(common::Info::Ok))?;
                send.write_all(msg.as_bytes()).await.unwrap();
                send.finish().await?;

                let a_conn = endp_lock.accept().await.unwrap().await?;
                let v_conn = endp_lock.accept().await.unwrap().await?;

                map_lock.insert(
                    name,
                    Client {
                        _conn: conn,
                        a_conn,
                        v_conn,
                    },
                );
            }
        }
        common::Message::Call(name) => {
            let mut lock = map.lock().await;
            let mut endp_lock = data_endpoint.lock().await;

            let msg;

            let contains = lock.contains_key(&name);
            //查看是否存在被呼叫用户
            if !contains {
                msg = common::Message::Result(common::Info::Err);
            } else {
                msg = common::Message::Result(common::Info::Ok);
            }

            let msg = serde_json::to_string(&msg)?;
            send.write_all(msg.as_bytes()).await?;
            send.finish().await?;

            if !contains {
                warn!("呼叫的用户({})不存在", name);
                return Ok(());
            }
            let a_conn = endp_lock.accept().await.unwrap().await?;
            let v_conn = endp_lock.accept().await.unwrap().await?;

            let ca = Client {
                _conn: conn,
                a_conn,
                v_conn,
            };
            let cb = lock.remove(&name).unwrap();

            drop(lock);

            handle_call::handle_call(ca, cb).await?;
        }
        // 请求等待呼叫用户列表
        common::Message::QueryUsers => {
            let lock = map.lock().await;
            let mut v = vec![];
            for e in lock.keys() {
                v.push(e.clone())
            }
            let buf =
                serde_json::to_string(&common::Message::Result(common::Info::UserList(v))).unwrap();
            send.write_all(buf.as_bytes()).await.unwrap();
        }
        _ => return Err(anyhow!("时序错误")),
    }
    Ok(())
}

mod handle_call;
