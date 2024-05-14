use anyhow::anyhow;

use quic::{Connection, Endpoint, SendStream};
use tracing::{info, warn};

use crate::{call::call, wait::wait, ClientMap};
use common::message::{Message, Res};

pub async fn handle_connection(
    conn: quic::Connecting,
    clients: ClientMap,
    data_endp: Endpoint,
) -> anyhow::Result<()> {
    // 首先建立连接
    let ctrl_conn = conn.await?;
    let client_addr = ctrl_conn.remote_address();
    info!("连接建立 remote_addr({})", client_addr);

    let stream = ctrl_conn.accept_bi().await;
    let stream = match stream {
        Ok(s) => s,
        Err(e) => {
            return Err(anyhow!("{}", e.to_string()));
        }
    };

    let (send, mut recv) = stream;

    // 读取第一个请求
    match recv.read_to_end(usize::MAX).await {
        Ok(data) => match serde_json::from_slice::<Message>(&data) {
            Ok(msg) => {
                //处理请求
                info!("请求: {}", msg);
                if let Err(e) = handle_req(msg, clients.clone(), send, data_endp, ctrl_conn).await {
                    warn!("{}", e.to_string())
                }
            }
            Err(e) => return Err(e.into()),
        },
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

async fn handle_req(
    msg: Message,
    clients: ClientMap,
    mut send: SendStream,
    data_endp: Endpoint,
    ctrl_conn: Connection,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        Message::Wait(name) => wait(name, clients, send, ctrl_conn).await?,
        Message::Call(name) => call(name, clients, send, data_endp, ctrl_conn).await?,
        // 请求等待呼叫用户列表
        Message::QueryUsers => {
            let clients_lock = clients.lock().await;
            let mut v = vec![];
            for e in clients_lock.keys() {
                v.push(e.clone())
            }

            let msg = Message::Response(Res::UserList(v));
            send.write_all(&msg.to_vec_u8()).await.unwrap();
            send.finish().await?;
            info!("查询结束")
        }
        Message::Hello => {
            send.write_all(&Message::Response(Res::Ok).to_vec_u8())
                .await?;
            send.finish().await?;
        }
        _ => return Err(anyhow!("时序错误")),
    }
    Ok(())
}
