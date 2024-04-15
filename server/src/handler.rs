use std::{sync::Arc, time::Duration};

use anyhow::anyhow;

use quic::{Connection, Endpoint, SendStream};
use tracing::{debug, error, info, warn};

use crate::{Client, ClientMap};

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
        Ok(data) => match serde_json::from_slice::<common::Message>(&data) {
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

#[allow(dead_code)]
async fn handle_req(
    msg: common::Message,
    clients: ClientMap,
    mut send: SendStream,
    data_endp: Endpoint,
    ctrl_conn: Connection,
) -> anyhow::Result<()> {
    match msg {
        // 挂线, 等待接听
        common::Message::Wait(name) => {
            let mut clients_lock = clients.lock().await;

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
                let a_conn = data_endp.accept().await.unwrap().await?;
                // 视频连接
                let v_conn = data_endp.accept().await.unwrap().await?;

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

                info!("音视频连接建立");
                info!("name({}) 加入等待接听列表", name);

                let ctrl = Arc::new(tokio::sync::Mutex::new(Some(ctrl_conn.clone())));
                let ctrl_c = ctrl.clone();
                let ctrl = Some(ctrl);
                // 客户端保活
                tokio::spawn(async move {
                    debug!("保活线程创建");
                    let wait = common::Message::Result(common::Info::Wait);
                    loop {
                        let ctrl_lock = ctrl_c.lock().await;
                        if ctrl_lock.is_none() {
                            break;
                        } else {
                            match ctrl_lock.clone().unwrap().open_bi().await {
                                Ok((mut send, _)) => {
                                    send.write_all(&wait.to_vec_u8()).await?;
                                    send.finish().await?;
                                    debug!("发送保活信息");
                                }
                                Err(e) => break error!("保活线程: {e}"),
                            }
                        }
                        drop(ctrl_lock);
                        tokio::time::sleep(Duration::from_millis(2000)).await;
                    }
                    debug!("保活线程退出");
                    Ok::<(), anyhow::Error>(())
                });

                clients_lock.insert(
                    name,
                    Client {
                        ctrl_conn,
                        a_conn,
                        v_conn,
                        ctrl,
                    },
                );
                info!("WAIT 请求处理完成")
            }
        }
        common::Message::Call(name) => {
            let mut clients_lock = clients.lock().await;

            debug!("获取锁");

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
            debug!("回传状态");

            if !contains {
                warn!("呼叫的用户({})不存在", name);
                return Ok(());
            }

            // 音频连接
            let a_conn = data_endp.accept().await.unwrap().await?;
            // 视频连接
            let v_conn = data_endp.accept().await.unwrap().await?;

            debug!("接收音视频连接");

            let c_active = Client {
                ctrl_conn,
                a_conn,
                v_conn,
                ctrl: None,
            };

            let c_passive = clients_lock.remove(&name).unwrap();
            // 停止等待
            let _ = c_passive.ctrl.clone().unwrap().lock().await.take();
            debug!("要求保活线程停止");

            drop(clients_lock);
            handle_call(c_active, c_passive).await?;
        }
        // 请求等待呼叫用户列表
        common::Message::QueryUsers => {
            let clients_lock = clients.lock().await;
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
    let (mut wake_sent, _) = passive.ctrl_conn.open_bi().await?;
    wake_sent.write_all(&msg.to_vec_u8()).await?;
    wake_sent.finish().await?;

    // 转发数据
    let t1 = tokio::spawn(exchange_uni(active.a_conn, passive.a_conn, "音频"));
    let t2 = tokio::spawn(exchange_uni(active.v_conn, passive.v_conn, "视频"));
    info!("转发音视频数据");
    let _ = tokio::join!(t1, t2);
    Ok(())
}

async fn exchange_uni(
    a: quic::Connection,
    b: quic::Connection,
    name: &'static str,
) -> anyhow::Result<()> {
    let a_c = a.clone();
    let b_c = b.clone();

    // a to b
    let fut1 = async move {
        let mut a2b = 0u32;

        loop {
            // match (a_c.accept_uni().await, b_c.open_uni().await) {
            //     (Ok(mut recv), Ok(mut send)) => {
            //         if let Ok(data) = recv.read_to_end(usize::MAX).await {
            //             if send.write_all(&data).await.is_err() || send.finish().await.is_err() {
            //                 break error!("send to b");
            //             }
            //         } else {
            //             break error!("read from a");
            //         }
            //     }
            //     (Ok(_), Err(e)) => break error!("b open {e}"),
            //     (Err(e), Ok(_)) => break error!("a acpt {e}"),
            //     (Err(_), Err(_)) => break error!("err"),
            // }

            match a_c.accept_uni().await {
                Ok(mut recv) => match b_c.open_uni().await {
                    Ok(mut send) => {
                        if let Ok(data) = recv.read_to_end(usize::MAX).await {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to b");
                            } else {
                                a2b += 1;
                                if a2b % 20 == 0 {
                                    debug!("转发包 {name}");
                                }
                            }
                        } else {
                            break error!("read from a");
                        }
                    }
                    Err(e) => break error!("b open {e}"),
                },
                Err(e) => break error!("a accept {e}"),
            }
        }
    };

    // p to a
    let fut2 = async move {
        let mut b2a = 0u32;

        loop {
            // match (b.accept_uni().await, a.open_uni().await) {
            //     (Ok(mut recv), Ok(mut send)) => {
            //         if let Ok(data) = recv.read_to_end(usize::MAX).await {
            //             if send.write_all(&data).await.is_err() || send.finish().await.is_err() {
            //                 break error!("send to a");
            //             }
            //         } else {
            //             break error!("read from b");
            //         }
            //     }
            //     (Ok(_), Err(e)) => break error!("a open {e}"),
            //     (Err(e), Ok(_)) => break error!("b acpt {e}"),
            //     (Err(_), Err(_)) => break error!("err"),
            // }

            match b.accept_uni().await {
                Ok(mut recv) => match a.open_uni().await {
                    Ok(mut send) => {
                        if let Ok(data) = recv.read_to_end(usize::MAX).await {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to a");
                            } else {
                                b2a += 1;
                                if b2a % 20 == 0 {
                                    debug!("转发包 {name}");
                                }
                            }
                        } else {
                            break error!("read from b");
                        }
                    }
                    Err(e) => break error!("a open {e}"),
                },
                Err(e) => break error!("b accept {e}"),
            }
        }
    };

    let t1 = tokio::spawn(fut1);
    let t2 = tokio::spawn(fut2);

    let _ = tokio::join!(t1, t2);
    info!("数据转发停止");
    Ok(())
}

async fn exchange_bi(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
    loop {
        match a.accept_bi().await {
            Ok((mut a_s, mut a_r)) => match b.accept_bi().await {
                Ok((mut b_s, mut b_r)) => {
                    let fut1 = async move {
                        match a_r.read_to_end(usize::MAX).await {
                            Ok(data) => {
                                if b_s.write_all(&data).await.is_err()
                                    || b_s.finish().await.is_err()
                                {
                                    error!("bs")
                                }
                            }
                            Err(e) => error!("ar {e}"),
                        }
                    };

                    let fut2 = async move {
                        match b_r.read_to_end(usize::MAX).await {
                            Ok(data) => {
                                if a_s.write_all(&data).await.is_err()
                                    || a_s.finish().await.is_err()
                                {
                                    error!("as")
                                }
                            }
                            Err(e) => error!("br {e}"),
                        }
                    };

                    let t1 = tokio::spawn(fut1);
                    let t2 = tokio::spawn(fut2);
                    let _ = tokio::join!(t1, t2);
                }
                Err(e) => break error!("b accept {e}"),
            },
            Err(e) => break error!("a accept {e}"),
        }
    }
    Ok(())
}

async fn exchange_uni_channel(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
    let (abs, mut abr) = tokio::sync::mpsc::channel::<Vec<u8>>(1024 * 200);
    let (bas, mut bar) = tokio::sync::mpsc::channel::<Vec<u8>>(1024 * 200);
    let a_c = a.clone();
    let b_c = b.clone();

    // read a
    let fut1 = async move {
        loop {
            match a_c.accept_uni().await {
                Ok(mut recv) => match recv.read_to_end(usize::MAX).await {
                    Ok(data) => {
                        if abs.send(data).await.is_err() {
                            break error!("abs send");
                        };
                    }
                    Err(_) => break error!("recv a"),
                },
                Err(e) => break error!("a accept {e}"),
            }
        }
    };

    let a_c = a.clone();

    // send a
    let fut2 = async move {
        loop {
            match a_c.open_uni().await {
                Ok(mut send) => {
                    match bar.recv().await {
                        Some(data) => {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to a");
                            }
                        }
                        None => break error!("bar recv"),
                    }
                    // abs.send(value)
                }
                Err(e) => break error!("a accept {e}"),
            }
        }
    };

    // read b
    let fut3 = async move {
        loop {
            match b_c.accept_uni().await {
                Ok(mut recv) => match recv.read_to_end(usize::MAX).await {
                    Ok(data) => {
                        if bas.send(data).await.is_err() {
                            break error!("bas send");
                        };
                    }
                    Err(_) => break error!("recv b"),
                },
                Err(e) => break error!("b accept {e}"),
            }
        }
    };
    let b_c = a.clone();

    // send b
    let fut4 = async move {
        loop {
            match b_c.open_uni().await {
                Ok(mut send) => {
                    match abr.recv().await {
                        Some(data) => {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to b");
                            }
                        }
                        None => break error!("abr recv"),
                    }
                    // abs.send(value)
                }
                Err(e) => break error!("b accept {e}"),
            }
        }
    };

    let t1 = tokio::spawn(fut1);
    let t2 = tokio::spawn(fut2);

    let t3 = tokio::spawn(fut3);
    let t4 = tokio::spawn(fut4);
    let _ = tokio::join!(t1, t2, t3, t4);
    info!("数据转发停止");
    Ok(())
}

#[allow(dead_code)]
async fn exchange_once_accept(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
    let (mut a_send, mut a_recv) = a.accept_bi().await?;
    debug!("收到a连接");
    let (mut b_send, mut b_recv) = b.accept_bi().await?;
    debug!("收到b连接");

    let mut abuf = vec![0u8; 1024];
    let mut bbuf = vec![0u8; 1024];
    let fut1 = async move {
        loop {
            match a_recv.read_exact(&mut abuf).await {
                Ok(_) => match b_send.write_all(&abuf).await {
                    Ok(_) => (),
                    Err(e) => break error!("{e} {}", line!()),
                },
                Err(e) => break error!("{e} {}", line!()),
            }
        }
    };

    let fut2 = async move {
        loop {
            match b_recv.read_exact(&mut bbuf).await {
                Ok(_) => match a_send.write_all(&bbuf).await {
                    Ok(_) => (),
                    Err(e) => break error!("{e} {}", line!()),
                },
                Err(e) => break error!("{e} {}", line!()),
            }
        }
    };

    let t1 = tokio::spawn(fut1);
    let t2 = tokio::spawn(fut2);
    let _ = tokio::join!(t1, t2);
    Ok(())
}

#[allow(dead_code)]
async fn transfer_(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
    // a to p
    let fut = async move {
        loop {
            // todo
            match (a.accept_bi().await, b.accept_bi().await) {
                (Ok((mut a_s, mut a_r)), Ok((mut b_s, mut b_r))) => {
                    match (
                        a_r.read_to_end(usize::MAX).await,
                        b_r.read_to_end(usize::MAX).await,
                    ) {
                        (Ok(ad), Ok(bd)) => {
                            if !(a_s.write_all(&bd).await.is_ok()
                                && a_s.finish().await.is_ok()
                                && b_s.write_all(&ad).await.is_ok()
                                && b_s.finish().await.is_ok())
                            {
                                break error!("数据转发失败");
                            }
                        }
                        (Ok(_), Err(_)) => todo!(),
                        (Err(_), Ok(_)) => todo!(),
                        (Err(_), Err(_)) => todo!(),
                    }
                }
                (Ok(_), Err(e)) => break error!("b accept: {e} {}", line!()),
                (Err(e), Ok(_)) => break error!("a accept: {e} {}", line!()),
                (Err(ea), Err(eb)) => break error!("accept: {ea} {eb} {}", line!()),
            }
        }
    };
    fut.await;

    info!("数据转发停止");
    Ok(())
}
