use std::net::SocketAddr;

use tracing::{debug, error, Level};

fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        // .with_env_filter("debug")
        .with_max_level(Level::DEBUG)
        .init();

    let mut buf = [0u8; 1024];
    let socket = std::net::UdpSocket::bind("0.0.0.0:12345".parse::<SocketAddr>().unwrap())
        .inspect_err(|e| error!("bind err {e}"))
        .unwrap();

    debug!("服务器监听地址: {}", socket.local_addr().unwrap());

    let (_, a_addr) = socket.recv_from(&mut buf).unwrap();
    debug!("取得客户端A的NAT映射地址: {}", a_addr);

    let (_, b_addr) = socket.recv_from(&mut buf).unwrap();
    debug!("取得客户端B的NAT映射地址: {}", b_addr);

    socket
        .send_to(a_addr.to_string().as_bytes(), b_addr)
        .unwrap();

    debug!("向客户端B发送客户端A的NAT映射地址");

    let (_, _) = socket.recv_from(&mut buf).unwrap();
    debug!("客户端B已发送数据包至客户端A");
    socket
        .send_to(b_addr.to_string().as_bytes(), a_addr)
        .unwrap();

    debug!("向客户端A发送客户端B的NAT映射地址");
    
}
