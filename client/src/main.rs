use std::{fs, net::SocketAddr, sync::Arc};

use clap::Parser;
use command::Cli;
use quic::Endpoint;


const FRAME_MSG_BYTE_SIZE: usize = 1382411;
const AUDIO_MSG_BYTE_SIZE: usize = 3851;

#[tokio::main]
async fn main() {
    let cli = command::Cli::parse();
    println!("{} {}", cli.addr, cli.name);
    let remote_addr: SocketAddr = cli.addr.parse().unwrap();

    let endpoint = match config(cli.clone()) {
        Ok(ept) => ept,
        Err(e) => {
            println!("err: {e} [{} {}]", file!(), line!());
            return;
        }
    };

    match cli.command {
        command::Commands::Wait => {
            let conn = match wait::wait(endpoint.clone(), remote_addr, &cli.server, &cli.name).await
            {
                Ok(ok) => ok,
                Err(err) => {
                    println!("err: {}", err.to_string());
                    return;
                }
            };
            //todo
            conn.close(0u8.into(), b"done");
            endpoint.wait_idle().await;
        }
        command::Commands::Call { name } => println!("call {}", name),
        command::Commands::Query => println!("query"),
    }
}

fn config(cli: Cli) -> anyhow::Result<Endpoint> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&rustls::Certificate(fs::read(&cli.cert)?))?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let client_config = quic::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quic::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}

mod call;
mod command;
mod wait;
