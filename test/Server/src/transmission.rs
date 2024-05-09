use std::{fs, sync::Arc};

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::make_endpoint;

pub async fn transmission() {
    let data = Arc::new(vec![0; 1024 * 10]);
    println!("{}", data.len());

    let endp = make_endpoint("0.0.0.0:12347".parse().unwrap()).unwrap();

    let datac = data.clone();
    let t1 = tokio::spawn(async move {
        loop {
            let data = datac.clone();
            let conn = endp.accept().await.unwrap().await.unwrap();
            tokio::spawn(async move {
                let e = async {
                    loop {
                        let s = conn.open_uni().await;
                        match s {
                            Ok(mut s) => {
                                // println!("send");
                                s.write_all(&data).await?;
                                s.finish().await?;
                            }
                            Err(_e) => {
                                // println!("{e}");
                                break anyhow::Ok(());
                            }
                        }
                    }
                }
                .await;
                println!("{:?}", e)
            });
        }
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
            let data = data.clone();
            let (stream, _) = listener.accept().await.unwrap();

            let mut tls_stream = acceptor.accept(stream).await.expect("Failed to accept");

            tokio::spawn(async move {
                let _ = async {
                    let e = async {
                        loop {
                            tls_stream.write_all(&data).await?;
                            if tls_stream.flush().await.is_err() {
                                break anyhow::Ok(());
                            };
                        }
                    }
                    .await;
                    println!("{e:?}")
                }
                .await;
            });
        }
    });

    let _e = tokio::join!(t1, t2);
}
