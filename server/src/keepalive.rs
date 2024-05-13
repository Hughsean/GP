use std::{sync::Arc, time::Duration};

use common::message::{Message, Res};
use quic::Connection;
use tracing::{debug, error, info};

use crate::ClientMap;

pub async fn fun(
    conn: Arc<tokio::sync::Mutex<Option<Connection>>>,
    clientsc: ClientMap,
    name: String,
) {
    debug!("保活线程创建");
    let wait = Message::Response(Res::Wait);
    let mut self_exit = false;
    loop {
        let ctrl_lock = conn.lock().await;
        if ctrl_lock.is_none() {
            break;
        } else {
            match ctrl_lock.clone().unwrap().open_bi().await {
                Ok((mut send, _)) => {
                    if let Err(_) = send.write_all(&wait.to_vec_u8()).await {
                        self_exit = true;
                        break;
                    }
                    if let Err(_) = send.finish().await {
                        self_exit = true;
                        break;
                    }
                    debug!("发送保活信息");
                }
                Err(e) => {
                    self_exit = true;
                    break error!("保活线程: {e}");
                }
            }
        }
        drop(ctrl_lock);
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
    if self_exit {
        if let Some(_) = clientsc.lock().await.remove(&name) {
            info!("客户端{name}主动退出等待");
        }
    }
    debug!("保活线程退出");
}
