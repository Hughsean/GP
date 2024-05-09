use std::cell::RefCell;
use std::rc::Rc;

use common::message;
use criterion::Criterion;
use quic::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_rustls::client::TlsStream;

const BUF_SIZE: usize = 1024;

#[derive(Clone, Copy)]
struct InputQuic<'a>(&'a Runtime, &'a Connection);

impl<'a> std::fmt::Display for InputQuic<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("quic")
    }
}

#[derive(Clone)]
struct InputTcp<'a>(&'a Runtime, Rc<RefCell<TlsStream<TcpStream>>>);

impl<'a> std::fmt::Display for InputTcp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tcp")
    }
}

pub fn quic_data_recv(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let endp = rt
        .block_on(async {
            let cert = {
                let cert = std::fs::read("cert/cert.der")?;
                let cert = rustls::Certificate(cert);
                cert
            };

            let addr = "0.0.0.0:0".parse()?;
            // let certs = vec![cert.clone()];
            let mut endpoint;

            let mut roots = rustls::RootCertStore::empty();
            roots.add(&cert)?;
            let client_crypto = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(roots)
                .with_no_client_auth();

            let client_config = quic::ClientConfig::new(std::sync::Arc::new(client_crypto));

            endpoint = quic::Endpoint::client(addr)?;
            endpoint.set_default_client_config(client_config);

            // let client_config;
            // Ok::<Endpoint, anyhow::Error>(endpoint)
            anyhow::Ok(endpoint)
        })
        .unwrap();

    let conn = rt.block_on(async {
        endp.connect("122.51.128.39:12347".parse().unwrap(), "localhost")
            .unwrap()
            .await
            .unwrap()
    });

    // let mut s = rt.block_on(conn.open_uni()).unwrap();

    let input = InputQuic(&rt, &conn);

    fn fun(ip: &InputQuic) {
        ip.0.block_on(async {
            let (mut s, mut r) = ip.1.open_bi().await.unwrap();
            // println!("\ns1");
            s.write_all(&message::Message::Hello.to_vec_u8())
                .await
                .unwrap();
            // println!("s2");
            s.finish().await.unwrap();
            // println!("s3");
            let _v = r.read_to_end(usize::MAX).await.unwrap();
            // println!("{}", v.len())
        });
    }

    
    c.bench_function("QUIC data req", |b| b.iter(|| fun(&input)));
}

pub fn tcp_data_recv(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // 加载CA证书
    let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();

    let cert = {
        let cert = std::fs::read("cert/cert.der").unwrap();
        tokio_rustls::rustls::Certificate(cert)
    };

    root_cert_store.add(&cert).unwrap();

    let config = tokio_rustls::rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let tls_stream = rt.block_on(async {
        let stream = tokio::net::TcpStream::connect("122.51.128.39:12348")
            .await
            .unwrap();

        let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config.clone()));
        let domain = tokio_rustls::rustls::ServerName::try_from("localhost").unwrap();

        connector
            .connect(domain, stream)
            .await
            .expect("Failed to connect")
    });

    let input = InputTcp(&rt, Rc::new(RefCell::new(tls_stream)));

    fn fun(input: InputTcp) {
        let mut buf = vec![0; BUF_SIZE];
        // let mut buf = buf;
        input.0.block_on(async move {
            let req = message::Message::Hello.to_vec_u8();

            let mut tls = input.1.borrow_mut();
            tls.write_u64(req.len() as u64).await.unwrap();
            tls.flush().await.unwrap();
            tls.write_all(&req).await.unwrap();
            tls.flush().await.unwrap();
            tls.read_exact(&mut buf).await.unwrap();
            // println!("{}", ip.2.len());
        });
    }

    c.bench_function("TLS over TCP data req", |b| b.iter(|| fun(input.clone())));
}
