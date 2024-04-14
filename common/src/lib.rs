use std::{fmt::Display, fs, net::SocketAddr, sync::Arc, time::Duration};

use quic::IdleTimeout;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Message {
    /// 请求在服务器挂起等待
    Wait(String),

    /// 请求呼叫
    Call(String),

    /// 请求服务器可被呼叫用户列表
    QueryUsers,

    /// 音频帧字节大小, 视频帧字节大小
    // FrameSize(usize, usize),

    /// 请求结束通话
    Close,

    /// 服务器回应请求结果
    Result(Info),
}

impl Message {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        let json = serde_json::to_vec(&self).unwrap();
        json
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Message::Wait(name) => format!("等待被呼叫 name({})", name),
            Message::Call(name) => format!("呼叫 name({})", name),
            Message::QueryUsers => "查询等待列表".into(),
            // Message::FrameSize(a, v) => format!("音频帧字节大小({a}) 视频帧字节大小({v})"),
            Message::Close => "关闭通信".into(),
            Message::Result(_) => "Result".into(),
        };
        f.write_str(&str)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Info {
    Ok,
    Err,
    Wait,
    Wake,
    UserList(Vec<String>),
}

impl Info {
    pub fn is_ok(&self) -> bool {
        match self {
            Info::Ok => true,
            _ => false,
        }
    }
}

pub enum EndpointType {
    Server(SocketAddr),
    Client(SocketAddr),
}

pub fn make_endpoint(enable: EndpointType) -> anyhow::Result<quic::Endpoint> {
    let (cert, key) = {
        let key = fs::read("cert/key.der")?;
        let cert = fs::read("cert/cert.der")?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);
        (cert, key)
    };

    let certs = vec![cert.clone()];
    let mut endpoint;

    match enable {
        EndpointType::Server(listen) => {
            let server_crypto = rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)?;
            let mut server_config = quic::ServerConfig::with_crypto(Arc::new(server_crypto));
            let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
            transport_config.max_concurrent_uni_streams(u16::MAX.into());
            // transport_config.max_concurrent_bidi_streams(value)
            // transport_config.keep_alive_interval(Some(Duration::from_millis(300)));
            // transport_config.max_idle_timeout(Some(IdleTimeout::try_from(Duration::from_secs(5))?));
            // transport_config.max_idle_timeout(None);
            endpoint = quic::Endpoint::server(server_config, listen)?;
        }
        EndpointType::Client(addr) => {
            let mut roots = rustls::RootCertStore::empty();
            roots.add(&cert)?;
            let client_crypto = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(roots)
                .with_no_client_auth();
            let client_config = quic::ClientConfig::new(Arc::new(client_crypto));

            endpoint = quic::Endpoint::client(addr)?;
            endpoint.set_default_client_config(client_config);
        }
    }
    // let client_config;
    Ok(endpoint)
}

#[test]
fn f() {
    let v: Vec<u8> = vec![0; 960];
    println!("{}", v.as_slice().len())
}

#[test]
fn f1() {
    let msg = Message::Call("(22)\011".into());
    let v = serde_json::to_string(&msg).unwrap();
    let v = v.as_bytes();
    let mut vv = vec![0u8; v.len() * 2];

    vv[..v.len()].copy_from_slice(v);

    let vv: Vec<_> = vv.into_iter().rev().collect();
    let vv: Vec<_> = vv.into_iter().skip_while(|e| *e == 0).collect();
    let vv: Vec<_> = vv.into_iter().rev().collect();

    let _msg: Message = serde_json::from_slice(&vv).unwrap();

    if let (Message::Call(s1), Message::Call(s2)) = (msg, _msg) {
        println!("{} {} {} {}", s1, s2, s1.len(), s2.len())
    }
}
