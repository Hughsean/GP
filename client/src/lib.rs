use common::endpoint_config::{make_endpoint, EndpointType};
use cpal::Stream;
use std::{fs, net::SocketAddr, sync::Arc};

use call::call;
use clap::Parser;
use command::Cli;

use quic::Endpoint;
use tracing::{error, info};

pub struct Audio {
    pub play: Stream,
    pub record: Stream,
}

pub struct App {
    /// 摄像头
    pub cam: Arc<opencv::videoio::VideoCapture>,
    /// 音频传入
    pub a_conn_in: quic::Connection,
    /// 音频传出
    pub a_conn_out: quic::Connection,
    /// 视频传入
    pub v_conn_in: quic::Connection,
    /// 视频传出
    pub v_conn_out: quic::Connection,
    /// 程序终止
    pub exit: Arc<tokio::sync::RwLock<bool>>,
}

#[test]
fn f() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let exit = Arc::new(tokio::sync::RwLock::new(false));

    let exitc = exit.clone();
    rt.spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let mut t = exitc.write().await;
        *t = true;
    });

    rt.block_on(async move {
        loop {
            if *exit.read().await {
                break;
            }
            println!("read");
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    });
}

#[allow(dead_code)]
async fn _main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("client=debug")
        .init();

    let cli = command::Cli::parse();
    let ctrl_addr: SocketAddr = cli
        .clone()
        .addr
        .unwrap_or("172.19.43.60:12345".into())
        .parse()
        .unwrap();

    let data_addr = SocketAddr::new(ctrl_addr.ip(), ctrl_addr.port() + 1);

    info!("ctrl_addr {}", ctrl_addr);
    info!("data_addr {}", data_addr);

    let endp = match config(cli.clone()) {
        Ok(ept) => ept,
        Err(e) => {
            println!("err: {e} [{} {}]", file!(), line!());
            return;
        }
    };

    let aendp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap();
    let vendp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap();

    match cli.command {
        command::Commands::Wait => {
            let conn = match wait::wait(
                endp.clone(),
                aendp,
                vendp,
                ctrl_addr,
                data_addr,
                &cli.server.unwrap_or("localhost".into()),
                &cli.name,
            )
            .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    error!("错误: {} line{}", err.to_string(), line!());
                    return;
                }
            };
            //todo

            conn.close(0u8.into(), b"done");
            endp.wait_idle().await;
        }
        command::Commands::Call { name } => match call(
            endp,
            aendp,
            vendp,
            ctrl_addr,
            data_addr,
            &cli.server.unwrap_or("localhost".into()),
            &name,
        )
        .await
        {
            Ok(_) => {
                info!("结束通话")
            }
            Err(e) => error!("{}", e.to_string()),
        },
        command::Commands::Query => println!("query"),
    }
}

fn config(cli: Cli) -> anyhow::Result<Endpoint> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&rustls::Certificate(fs::read(
        &cli.cert.unwrap_or("cert/cert.der".into()),
    )?))?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let client_config = quic::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quic::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}

mod audio;
mod call;
mod command;
mod video;
mod wait;
