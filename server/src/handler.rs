use anyhow::anyhow;
use quic::{RecvStream, SendStream};

use crate::{Client, ClientMap};

pub async fn handle_connection(conn: quic::Connecting, map: ClientMap) -> anyhow::Result<()> {
    let connection = conn.await?;
    async {
        println!("连接建立");
        // 只使用一个双向流
        loop {
            //接收流
            let stream = connection.accept_bi().await;

            let stream = match stream {
                Err(quic::ConnectionError::ApplicationClosed { .. }) => {
                    println!("连接关闭");
                    return Ok(());
                }
                Err(e) => {
                    return Err(anyhow!("{}", e.to_string()));
                }
                Ok(s) => s,
            };

            let (send, mut recv) = stream;
            // 读取请求

            match recv.read_to_end(usize::MAX).await {
                Ok(data) => match serde_json::from_slice::<common::Message>(&data) {
                    Ok(msg) => {
                        //处理请求
                        println!("into _req");
                        if let Err(e) =
                            handle_req(msg, map.clone(), send, recv, connection.clone()).await
                        {
                            println!("{}({}): {}", file!(), line!(), e.to_string())
                        }
                    }
                    Err(e) => println!("{}({}): {}", file!(), line!(), e.to_string()),
                },
                Err(e) => println!("{}({}): {}", file!(), line!(), e.to_string()),
            };
        }
    }
    .await?;

    Ok(())
}

#[allow(dead_code)]
async fn handle_req(
    msg: common::Message,
    map: ClientMap,
    mut send: SendStream,
    mut _recv: RecvStream,
    conn: quic::Connection,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        common::Message::Wait(s) => {
            let mut lock = map.lock().await;

            if lock.contains_key(&s) {
                // 用户名重复
                let msg = serde_json::to_string(&common::Message::Result(common::Info::Err))?;
                send.write_all(msg.as_bytes()).await.unwrap();
            } else {
                // OK
                let msg = serde_json::to_string(&common::Message::Result(common::Info::Ok))?;

                send.write_all(msg.as_bytes()).await.unwrap();
                println!("wait 回传");
                println!("{} 等待接听会话", &s);
                lock.insert(s, Client { conn });
            }
            send.finish().await.unwrap();
        }
        common::Message::Call(s) => {
            let mut lock = map.lock().await;
            let msg;

            let contains = lock.contains_key(&s);
            //查看是否存在被呼叫用户
            if !contains {
                msg = common::Message::Result(common::Info::Err);
            } else {
                msg = common::Message::Result(common::Info::Ok);
            }
            let msg = serde_json::to_string(&msg).unwrap();
            send.write_all(msg.as_bytes()).await.unwrap();
            send.finish().await.unwrap();

            if !contains {
                return Ok(());
            }

            let (_k, ca) = lock.remove_entry(&s).unwrap();
            drop(lock);
            let cb = Client { conn };

            handle_call::handle_call(ca, cb).await?;
            // 初始化用户呼叫的流
            // let mut user1send = v.send;
            // let mut user1revc = v.recv;

            // let mut user2send = send;
            // let mut user2revc = recv;

            // let fut1 = async move {
            //     loop {
            //         match user1revc.read_to_end(usize::MAX).await {
            //             Ok(data) => match handle_data(data, &mut user2send).await {
            //                 Ok(_) => (),
            //                 Err(_) => break,
            //             },
            //             Err(_) => break,
            //         }
            //     }
            // };
            // let fut2 = async move {
            //     loop {
            //         match user2revc.read_to_end(usize::MAX).await {
            //             Ok(data) => match handle_data(data, &mut user1send).await {
            //                 Ok(_) => (),
            //                 Err(_) => break,
            //             },
            //             Err(_) => break,
            //         }
            //     }
            // };

            // let t1 = tokio::spawn(fut1);
            // let t2 = tokio::spawn(fut2);
            // let _ = tokio::join!(t1, t2);
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
        _ => {
            todo!()
        }
    }
    Ok(())
}

mod handle_call;
