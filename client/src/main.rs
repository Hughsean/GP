use std::{fs, net::SocketAddr, sync::Arc};

use call::call;
use clap::Parser;
use command::Cli;

use log::{error, info};
use quic::Endpoint;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "client=DEBUG");
    env_logger::init();

    let cli = command::Cli::parse();
    // println!("{} {}", cli.addr, cli.name);
    let ctrl_addr: SocketAddr = cli
        .clone()
        .addr
        .unwrap_or("122.51.128.39:12345".into())
        .parse()
        .unwrap();
    let data_addr = SocketAddr::new(ctrl_addr.ip(), ctrl_addr.port() + 1);

    let ctrl_endp = match config(cli.clone()) {
        Ok(ept) => ept,
        Err(e) => {
            println!("err: {e} [{} {}]", file!(), line!());
            return;
        }
    };

    let data_endp =
        common::make_endpoint(common::EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap();

    match cli.command {
        command::Commands::Wait => {
            let conn = match wait::wait(
                ctrl_endp.clone(),
                data_endp.clone(),
                ctrl_addr,
                data_addr,
                &cli.server.unwrap_or("localhost".into()),
                &cli.name,
            )
            .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    println!("错误: {}", err.to_string());
                    return;
                }
            };
            //todo

            conn.close(0u8.into(), b"done");
            ctrl_endp.wait_idle().await;
        }
        command::Commands::Call { name } => match call(
            ctrl_endp,
            data_endp,
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
