use tracing::{debug, error, info};

pub async fn exchange_uni(
    a: quic::Connection,
    b: quic::Connection,
    name: &'static str,
) -> anyhow::Result<()> {
    let a_c = a.clone();
    let b_c = b.clone();
    const DISPALY_N: u64 = 200;

    // a to b
    let fut1 = async move {
        let mut a2b = 0u64;

        loop {
            match a_c.accept_uni().await {
                Ok(mut recv) => match b_c.open_uni().await {
                    Ok(mut send) => {
                        if let Ok(data) = recv.read_to_end(usize::MAX).await {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to b");
                            } else {
                                if a2b % DISPALY_N == 0 {
                                    debug!("转发包 {name}");
                                }
                                a2b += 1;
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

    // b to a
    let fut2 = async move {
        let mut b2a = 0u64;

        loop {
            match b.accept_uni().await {
                Ok(mut recv) => match a.open_uni().await {
                    Ok(mut send) => {
                        if let Ok(data) = recv.read_to_end(usize::MAX).await {
                            if send.write_all(&data).await.is_err() || send.finish().await.is_err()
                            {
                                break error!("send to a");
                            } else {
                                if b2a % DISPALY_N == 0 {
                                    debug!("转发包 {name}");
                                }
                                b2a += 1;
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
async fn transfer_dispared(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
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
