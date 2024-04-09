use anyhow::anyhow;
use quic::{RecvStream, SendStream};

use crate::{Client, ClientMap};

pub(crate) async fn handle_connection(
    conn: quic::Connecting,
    map: ClientMap,
) -> anyhow::Result<()> {
    let connection = conn.await?;
    async {
        println!("连接建立");
        // 只使用一个双向流
        // loop {
        //接收流
        let stream = connection.accept_bi().await;

        let stream = match stream {
            Err(quic::ConnectionError::ApplicationClosed { .. }) => {
                println!("连接关闭");
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
            Ok(s) => s,
        };

        let (send, mut recv) = stream;
        // 读取请求
        match recv.read_to_end(usize::MAX).await {
            Ok(data) => match serde_json::from_slice::<common::Message>(&data) {
                Ok(msg) => {
                    //处理请求
                    if let Err(e) = handle_req(msg, map, send, recv).await {
                        println!("{}({}): {}", file!(), line!(), e.to_string())
                    }
                }
                Err(e) => println!("{}({}): {}", file!(), line!(), e.to_string()),
            },
            Err(e) => println!("{}({}): {}", file!(), line!(), e.to_string()),
        };
        // }
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
    recv: RecvStream,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        common::Message::Wait(s) => {
            let mut lock = map.lock().await;

            if lock.contains_key(&s) {
                let buf = serde_json::to_string(&common::Message::Result(
                    common::Info::UserNameExisted,
                    "用户已存在".into(),
                ))
                .unwrap();
                send.write_all(buf.as_bytes()).await.unwrap();
            } else {
                lock.insert(s, Client { send, recv });
            }
        }
        common::Message::Call(s) => {
            let mut lock = map.lock().await;

            //查看是否存在被呼叫用户
            if !lock.contains_key(&s) {
                let buf = serde_json::to_string(&common::Message::Result(
                    common::Info::UserNameNotFound,
                    "用户不存在".into(),
                ))
                .unwrap();
                send.write_all(buf.as_bytes()).await.unwrap();
            }

            let (_k, v) = lock.remove_entry(&s).unwrap();
            drop(lock);

            // 初始化用户呼叫的流
            let mut user1send = v.send;
            let mut user1revc = v.recv;

            let mut user2send = send;
            let mut user2revc = recv;

            let fut1 = async move {
                loop {
                    match user1revc.read_to_end(usize::MAX).await {
                        Ok(data) => match handle_data(data, &mut user2send).await {
                            Ok(_) => (),
                            Err(_) => break,
                        },
                        Err(_) => break,
                    }
                }
            };
            let fut2 = async move {
                loop {
                    match user2revc.read_to_end(usize::MAX).await {
                        Ok(data) => match handle_data(data, &mut user1send).await {
                            Ok(_) => (),
                            Err(_) => break,
                        },
                        Err(_) => break,
                    }
                }
            };

            let _ = tokio::spawn(fut1).await;
            let _ = tokio::spawn(fut2).await;
        }
        // 请求等待呼叫用户列表
        common::Message::QueryUsers => {
            let lock = map.lock().await;
            let mut v = vec![];
            for e in lock.keys() {
                v.push(e.clone())
            }
            let buf = serde_json::to_string(&common::Message::Result(
                common::Info::UserList(v),
                "查询结果".into(),
            ))
            .unwrap();
            send.write_all(buf.as_bytes()).await.unwrap();
        }
        _ => {
            todo!()
        }
    }
    Ok(())
}

async fn handle_data(data: Vec<u8>, send: &mut SendStream) -> anyhow::Result<()> {
    let msg: common::Message = serde_json::from_slice(&data).unwrap();
    match msg {
        common::Message::Video { .. } | common::Message::Audio { .. } => {
            match send.write_all(&data).await {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("")),
            }
        }
        common::Message::Close => Err(anyhow!("Done")),
        _ => todo!(),
    }
}
