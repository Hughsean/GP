use std::time::Instant;

use criterion::{black_box, Criterion};
use quic::Endpoint;

pub fn quic_connection(c: &mut Criterion) {
    fn fun(endp: &Endpoint, rt: &tokio::runtime::Runtime) {
        rt.block_on(async {
            match endp.connect("122.51.128.39:12345".parse().unwrap(), "localhost") {
                Ok(c) => {
                    let _con = c.await.unwrap();
                }
                Err(_) => {
                    // let _conn = c.await.unwrap();
                }
            }
        });
    }

    c.bench_function("QUIC", move |b| {
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

            let start = Instant::now();
            for _ in 0..i {
                black_box(fun(&endp, &rt));
            }
            start.elapsed()
        })
    });
}

pub fn tcp_connection(c: &mut Criterion) {
    fn fun(rt: &tokio::runtime::Runtime, cfg: &tokio_rustls::rustls::ClientConfig) {
        rt.block_on(async {
            let stream = tokio::net::TcpStream::connect("122.51.128.39:12346")
                .await
                .unwrap();

            let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(cfg.clone()));
            let domain = tokio_rustls::rustls::ServerName::try_from("localhost").unwrap();

            let _tls_stream = connector
                .connect(domain, stream)
                .await
                .expect("Failed to connect");
        });
    }

    c.bench_function("TLS over TCP", move |b| {
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

            let start = Instant::now();
            for _i in 0..i {
                black_box(fun(&rt, &config));
            }
            start.elapsed()
        })
    });
}
