use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sendp = common::make_endpoint(common::EndpointType::Server("0.0.0.0:12345".parse()?))?;
    let cendpa = common::make_endpoint(common::EndpointType::Client("0.0.0.0:0".parse()?))?;
    let cendpb = common::make_endpoint(common::EndpointType::Client("0.0.0.0:0".parse()?))?;
    let _buf = [0u8; 2 * 1024];

    let t = tokio::spawn(async move {
        let ca = sendp.accept().await.unwrap().await.unwrap();
        let _caa = sendp.accept().await.unwrap().await.unwrap();
        let _caa = sendp.accept().await.unwrap().await.unwrap();

        let cb = sendp.accept().await.unwrap().await.unwrap();
        exchange(ca, cb).await.unwrap();
    });

    let ca = cendpa
        .connect("127.0.0.1:12345".parse()?, "localhost")
        .unwrap()
        .await?;

    let _caa = cendpa
        .connect("127.0.0.1:12345".parse()?, "localhost")
        .unwrap()
        .await?;
    let _caaa = cendpa
    .connect("127.0.0.1:12345".parse()?, "localhost")
    .unwrap()
    .await?;
    let cb = cendpb
        .connect("127.0.0.1:12345".parse()?, "localhost")
        .unwrap()
        .await?;

    let tt1 = tokio::spawn(async move {
        let sbuf = [0u8; 1024];
        let (mut s, mut r) = ca.open_bi().await.unwrap();
        let t1 = tokio::spawn(async move {
            loop {
                s.write_all(&sbuf).await.unwrap();
                sleep(Duration::from_secs(1)).await;
            }
        });

        let mut rbuf = [0u8; 1024];
        let t2 = tokio::spawn(async move {
            loop {
                r.read_exact(&mut rbuf).await.unwrap();
                sleep(Duration::from_secs(1)).await;
            }
        });
        let _ = tokio::join!(t1, t2);
    });

    let tt2 = tokio::spawn(async move {
        let sbuf = [0u8; 1024];
        let (mut s, mut r) = cb.open_bi().await.unwrap();
        let t1 = tokio::spawn(async move {
            loop {
                s.write_all(&sbuf).await.unwrap();
                sleep(Duration::from_secs(1)).await;
            }
        });

        let mut rbuf = [0u8; 1024];
        let t2 = tokio::spawn(async move {
            loop {
                r.read_exact(&mut rbuf).await.unwrap();
                sleep(Duration::from_secs(1)).await;
            }
        });
        let _ = tokio::join!(t1, t2);
    });
    let _ = t.await;
    let _ = tt1.await;
    let _ = tt2.await;

    Ok(())
}

async fn exchange(a: quic::Connection, b: quic::Connection) -> anyhow::Result<()> {
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
                    Ok(_) => println!("a 2 b"),
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
                    Ok(_) => println!("b 2 a"),
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
