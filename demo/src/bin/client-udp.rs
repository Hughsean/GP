use anyhow::anyhow;
fn main() {
    if let Err(e) = run() {
        println!("{}", e.to_string())
    }
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    // let addr: SocketAddr = "0.0.0.0:0".parse().unwrap();

    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;

    // let sck = Socket::new(
    //     Domain::for_address("0.0.0.0:12349".parse()?),
    //     socket2::Type::DGRAM,
    //     Some(socket2::Protocol::UDP),
    // )?;
    // sck.set_reuse_address(true)?;

    // sck.bind(&addr.into())?;

    // let socket: UdpSocket = sck.into();
    // let mut buf = vec![0u8;4];
    // sck.connect(&socket2::SockAddr::new(storage, len))?;
    // sck.c
    // let socket=sck.into
    socket.connect("122.51.128.39:12349")?;
    socket.send("".as_bytes())?;
    let addr = socket.local_addr()?.to_string();
    println!("wait,{}", &addr);

    let mut buf = vec![0u8; 4];
    let (_, recvaddr) = socket.recv_from(&mut buf)?;
    println!("{}",recvaddr);

    // drop(socket);

    // let endp = common::make_endpoint(common::EndpointType::Server(addr.parse()?))?;
    // // endp.rebind(std::net::UdpSocket::bind(&addr)?)?;

    // let c = endp.accept().await.ok_or(anyhow!("none"))?;
    // let c = c.await?;
    // println!("{}", c.remote_address());

    Ok(())
}
