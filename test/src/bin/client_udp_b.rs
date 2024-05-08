// use apps::common;

// use tokio::time::sleep;

use std::net::SocketAddr;

fn main() {
    if let Err(e) = run() {
        println!("{}", e.to_string())
    }
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;

    let mut buf = vec![0u8; 1024];

    let server_addr: SocketAddr = "122.51.128.39:12349".parse()?;
    socket.connect(server_addr)?;
    socket.send("hello".as_bytes())?;

    let (len, server_addr) = socket.recv_from(&mut buf)?;
    println!("{} {}", len, server_addr);

    let client_addr = String::from_utf8(buf[..len].to_vec())?;
    println!("{}", &client_addr);

    socket.connect(client_addr.parse::<SocketAddr>()?)?;
    // B 先向 A 发送数据
    socket.send(socket.local_addr()?.to_string().as_bytes())?;
    println!("--> A");
    // B 通知服务器
    socket.connect(server_addr)?;
    socket.send("hello".as_bytes())?;
    println!("--> S");
    
    socket.connect(client_addr)?;
    let (len, peer) = socket.recv_from(&mut buf)?;
    println!("from A len{}  {}", len, peer);

    Ok(())
}
