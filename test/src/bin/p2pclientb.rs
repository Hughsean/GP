// use apps::common;

// use tokio::time::sleep;

use std::net::SocketAddr;

use tracing::{debug, error, Level};

fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_max_level(Level::DEBUG)
        .init();

    if let Err(e) = run() {
        error!("{}", e.to_string())
    }
}

fn run() -> anyhow::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    let mut buf = vec![0u8; 1024];
    let server_addr: SocketAddr = "122.51.128.39:12345".parse()?;

    socket.connect(server_addr)?;
    socket.send("hello".as_bytes())?;
    debug!("连接服务器");

    let (len, server_addr) = socket.recv_from(&mut buf)?;
    let client_addr = String::from_utf8(buf[..len].to_vec())?;
    debug!("获得客户端A的NAT地址: {}", client_addr);

    socket.connect(client_addr.parse::<SocketAddr>()?)?;
    // B 先向 A 发送数据
    socket.send(socket.local_addr()?.to_string().as_bytes())?;
    debug!("向客户端A的NAT地址发送UDP数据包");

    // B 通知服务器
    socket.connect(server_addr)?;
    socket.send("hello".as_bytes())?;
    debug!("通知服务器 \"B已发送数据包\"");

    debug!("等待接收来自A的数据包");
    socket.connect(client_addr)?;
    let (len, peer) = socket.recv_from(&mut buf)?;
    debug!("from A len{}  {}", len, peer);

    Ok(())
}
