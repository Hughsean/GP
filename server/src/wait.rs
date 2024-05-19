use std::sync::Arc;

use common::message::{Message, Res};
use quic::{Connection, SendStream};
use tracing::{debug, info};

use crate::{guard::guard, Client, ClientMap};

pub async fn wait(
    name: String,
    clients: ClientMap,
    mut send: SendStream,
    ctrl_conn: Connection,
) -> anyhow::Result<()> {
    let mut clients_lock = clients.write().await;

    debug!("获取用户字典锁");
    if clients_lock.contains_key(&name) {
        // 用户名重复
        info!("用户名({})重复", name);

        let msg = Message::Response(Res::Err);
        send.write_all(&msg.to_vec_u8()).await.unwrap();
        send.finish().await?;
        Ok(())
    } else {
        let msg = Message::Response(Res::Ok);
        send.write_all(&msg.to_vec_u8()).await.unwrap();
        send.finish().await?;

        // // 音频连接
        // let a_conn = data_endp.accept().await.unwrap().await?;
        // // 视频连接
        // let v_conn = data_endp.accept().await.unwrap().await?;

        // 'test: {
        //     let data = [0u8; 1024 * 10];
        //     let mut n = 0;
        //     loop {
        //         let (mut s, _) = v_conn.accept_bi().await?;
        //         s.write_all(&data).await?;
        //         s.finish().await?;
        //         info!("send");
        //         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        //         n += 1;
        //         if n == 10 {
        //             break;
        //         }
        //     }
        // }

        info!("name({}) 加入等待接听列表", name);

        let ctrl = Arc::new(tokio::sync::RwLock::new(Some(ctrl_conn.clone())));
        let ctrl_c = ctrl.clone();
        let ctrl = Some(ctrl);
        let clientsc = clients.clone();
        let namec = name.clone();
        // 客户端保活
        tokio::spawn(guard(ctrl_c, clientsc, namec));
        debug!("创建守护任务发送线程池");
        clients_lock.insert(
            name.clone(),
            Client {
                ctrl_conn,
                a_conn: None,
                v_conn: None,
                ctrl,
            },
        );
        debug!("用户{}信息写入用户字典", name);
        info!("WAIT 请求处理完成");
        Ok(())
    }
}
