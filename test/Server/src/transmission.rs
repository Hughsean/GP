use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::{make_endpoint, Message};

const BUF_SIZE: usize = 1024 * 10;
pub async fn transmission() {
    // let mut buf = [0u8; BUF_SIZE];
    let endp = make_endpoint("0.0.0.0:12347".parse().unwrap()).unwrap();

    let t1 = tokio::spawn(async move {
        let buf = [0u8; BUF_SIZE];

        let conn = endp.accept().await.unwrap().await.unwrap();
        tokio::spawn(async move {
            loop {
                let t = conn.accept_bi().await;

                if let Ok((mut s, mut r)) = t {
                    let recv = r.read_to_end(usize::MAX).await.unwrap();

                    let req = serde_json::from_slice::<Message>(&recv).unwrap();

                    if let Message::Hello = req {
                        s.write_all(&buf).await.unwrap();

                        s.finish().await.unwrap();
                    }
                } else {
                    break println!("Err");
                };
                // anyhow::Ok(())
            }
        });
    });

    let addr = "0.0.0.0:12348".to_string();
    let listener = TcpListener::bind(&addr).await.unwrap();

    // 加载服务器证书和密钥
    let (certs, key) = {
        let key = fs::read("cert/key.der").unwrap();
        let cert = fs::read("cert/cert.der").unwrap();
        let key = tokio_rustls::rustls::PrivateKey(key);
        let cert = tokio_rustls::rustls::Certificate(cert);
        (cert, key)
    };

    let config = tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![certs], key)
        .expect("failed to create server config");

    let acceptor = TlsAcceptor::from(std::sync::Arc::new(config));

    // let data=data.clone();

    let t2 = tokio::spawn(async move {
        loop {
            // let data = data.clone();
            let (stream, _) = listener.accept().await.unwrap();

            let mut tls_stream = acceptor.accept(stream).await.expect("Failed to accept");

            tokio::spawn(async move {
                let mut buf = [0u8; BUF_SIZE];

                // let _ = async {
                loop {
                    // if tls_stream.write_all(&buf).await.is_err() {
                    //     break;
                    // }
                    let len = tls_stream.read_u64().await;
                    if let Ok(len) = len {
                        let _s = tls_stream
                            .read_exact(&mut buf[..len as usize])
                            .await
                            .unwrap();

                        let req = serde_json::from_slice::<Message>(&buf[..len as usize]).unwrap();

                        if let Message::Hello = req {
                            tls_stream.write_all(&buf).await.unwrap();
                            tls_stream.flush().await.unwrap();
                        }
                    } else {
                        break println!("tcp exit");
                    }
                }
                // }
                // .await;
            });
        }
    });

    let _e = tokio::join!(t1, t2);
}
