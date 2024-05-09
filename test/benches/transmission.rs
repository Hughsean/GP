use std::time::Instant;

use criterion::{black_box, Criterion};
use quic::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

const BUF_SIZE: usize = 1024 * 50;

#[allow(dead_code)]
pub fn quic_data_recv(c: &mut Criterion) {
    fn fun(conn: &Connection, rt: &tokio::runtime::Runtime, buf: &[u8]) {
        rt.block_on(async {
            let mut s = conn.open_uni().await.unwrap();
            // println!("s");
            s.write_all(buf).await.unwrap();
            s.finish().await.unwrap();
            // r.read_to_end(usize::MAX).await.unwrap();
        });
    }

    c.bench_function("QUIC data recv", move |b| {
        b.iter_custom(|i| {
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

            let buf = [0u8; BUF_SIZE];

            let start = Instant::now();
            for _e in 0..i {
                black_box(fun(&conn, &rt, &buf));
            }
            start.elapsed()
        })
    });
}

#[allow(dead_code)]
pub fn tcp_data_recv(c: &mut Criterion) {
    fn fun(
        rt: &tokio::runtime::Runtime,
        tls_stream: &mut TlsStream<TcpStream>,
        mut buf: &mut [u8],
    ) {
        // let mut buf = buf;
        rt.block_on(async move {
            tls_stream.read_exact(&mut buf).await.unwrap();
            tls_stream.flush().await.unwrap();
        });
    }

    c.bench_function("TLS over TCP data recv", move |b| {
        b.iter_custom(|i| {
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

            let mut s = rt.block_on(async {
                let stream = tokio::net::TcpStream::connect("122.51.128.39:12348")
                    .await
                    .unwrap();

                let connector =
                    tokio_rustls::TlsConnector::from(std::sync::Arc::new(config.clone()));
                let domain = tokio_rustls::rustls::ServerName::try_from("localhost").unwrap();

                connector
                    .connect(domain, stream)
                    .await
                    .expect("Failed to connect")
            });
            let mut buf = [0; BUF_SIZE];
            let start = Instant::now();
            for _e in 0..i {
                // println!("{}", e);
                black_box(fun(&rt, &mut s, &mut buf));
            }
            start.elapsed()
        })
    });
}
