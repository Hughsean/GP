use std::fs;

use tokio::net::TcpListener;
// use tokio_rustls::rustls::{self};
use tokio_rustls::TlsAcceptor;

use crate::make_endpoint;

pub async fn connection() {
    let endp = make_endpoint("0.0.0.0:12345".parse().unwrap()).unwrap();

    let mut q = std::collections::VecDeque::new();
    let t1 = tokio::spawn(async move {
        loop {
            let e = endp.accept().await.unwrap().await;
            if let Ok(c) = e {
                q.push_back(c);
            }
        }
    });

    let addr = "0.0.0.0:12346".to_string();
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

    let t2 = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();

            let _tls_stream = acceptor.accept(stream).await.expect("Failed to accept");
        }
    });

    let _e = tokio::join!(t1, t2);
}
