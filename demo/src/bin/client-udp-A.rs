// use apps::common;

// use tokio::time::sleep;

use std::{net::SocketAddr, thread::sleep, time::Duration};

fn main() {
    if let Err(e) = run() {
        println!("{}", e.to_string())
    }
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;

    let mut buf = vec![0u8; 1024];

    let addr: SocketAddr = "122.51.128.39:12348".parse()?;
    socket.connect(addr)?;
    socket.send("hello".as_bytes())?;

    let (len, server_addr) = socket.recv_from(&mut buf)?;
    println!("{} {}", len, server_addr);

    let client_addr = String::from_utf8(buf[..len].to_vec())?;
    println!("from S {}", &client_addr);

    socket.connect(client_addr.parse::<SocketAddr>()?)?;
    // let len = socket.recv(&mut buf)?;
    sleep(Duration::from_secs(1));
    socket.send("hello".as_bytes())?;
    println!("--> B");
    socket.send(socket.local_addr()?.to_string().as_bytes())?;
    println!("{}", len);

    Ok(())
}
