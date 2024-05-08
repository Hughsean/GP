use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quic::Endpoint;

fn quic_connection(c: &mut Criterion) {
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

            let endp = rt.block_on(async {
                common::endpoint_config::make_endpoint(
                    common::endpoint_config::EndpointType::Client("0.0.0.0:0".parse().unwrap()),
                )
                .unwrap()
            });

            let start = Instant::now();
            for _ in 0..i {
                black_box(fun(&endp, &rt));
            }
            start.elapsed()
        })
    });
}

fn tcp_connection(c: &mut Criterion) {
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

criterion_group!(benches, quic_connection, tcp_connection);
criterion_main!(benches);
