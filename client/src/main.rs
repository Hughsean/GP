use std::{fs, net::SocketAddr, sync::Arc};

use clap::Parser;
use command::Cli;
use quic::Endpoint;

fn main() {
    let cli = command::Cli::parse();
    println!("{} {}", cli.addr, cli.name);
    match cli.command {
        command::Commands::Wait => println!("wait"),
        command::Commands::Call { name } => println!("call {}", name),
        command::Commands::Query => println!("query"),
    }
    let remote_addr: SocketAddr = cli.addr.parse().unwrap();
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

mod command;
mod wait;