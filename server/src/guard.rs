use std::{sync::Arc, time::Duration};

use common::message::{Message, Res};
use quic::Connection;
use tracing::{debug, error, info};

use crate::ClientMap;

pub async fn guard(
    conn: Arc<tokio::sync::RwLock<Option<Connection>>>,
    clientsc: ClientMap,
    name: String,
) {
    debug!("守护线程创建");
    let wait = Message::Response(Res::Wait);
    let exit = async {
        loop {
            let ctrl_lock = conn.read().await;
            if ctrl_lock.is_none() {
                debug!("用户{name}被呼叫, 守卫任务退出");
                break anyhow::Ok(());
            } else {
                match ctrl_lock.clone().unwrap().open_bi().await {
                    Ok((mut send, _)) => {
                        send.write_all(&wait.to_vec_u8()).await?;
                        // {
                        //     self_exit = true;
                        //     break;
                        // }
                        // if let Err(_) =
                        send.finish().await?;
                        // {
                        //     self_exit = true;
                        //     break;
                        // }
                        // debug!("发送保活信息");
                    }
                    Err(e) => {
                        // self_exit = true;
                        break {
                            error!("保活线程: {e}");
                            Err(e.into())
                        };
                    }
                }
            }
            drop(ctrl_lock);
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
    }
    .await;

    if exit.is_err() {
        debug!("报文发送失败, 守卫任务退出");
        info!("用户{name}主动退出等待");
        if let Some(_) = clientsc.write().await.remove(&name) {
            debug!("从用户字典移除用户{name}信息");
        }
    }
    info!("守护线程退出");
}
