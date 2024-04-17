// use apps::common;

// use tokio::time::sleep;

use std::net::SocketAddr;

use tracing::{debug, Level};

fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_max_level(Level::DEBUG)
        .init();
    if let Err(e) = run() {
        println!("{}", e.to_string())
    }
}

fn run() -> anyhow::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;

    let mut buf = vec![0u8; 1024];

    let addr: SocketAddr = "122.51.128.39:12345".parse()?;
    socket.connect(addr)?;
    socket.send("hello".as_bytes())?;
    debug!("连接服务器");

    let (len, _server_addr) = socket.recv_from(&mut buf)?;
    let client_addr = String::from_utf8(buf[..len].to_vec())?;
    debug!("获得客户端B的NAT地址: {}", client_addr);

    socket.connect(client_addr.parse::<SocketAddr>()?)?;
    socket.send("hello".as_bytes())?;
    debug!("向客户端B的NAT地址发送UDP数据包");

    // socket.send(socket.local_addr()?.to_string().as_bytes())?;
    // println!("");
    // println!("{}", len);

    Ok(())
}
