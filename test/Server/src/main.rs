use connection::connection;
use std::{fs, net::SocketAddr, sync::Arc};
use transmission::transmission;
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tokio::join!(connection(),transmission());
}

mod connection;
mod transmission;

pub fn make_endpoint(addr: SocketAddr) -> anyhow::Result<quic::Endpoint> {
    let (cert, key) = {
        let key = fs::read("cert/key.der")?;
        let cert = fs::read("cert/cert.der")?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);
        (cert, key)
    };

    let certs = vec![cert.clone()];

    let server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let mut server_config = quic::ServerConfig::with_crypto(Arc::new(server_crypto));
    // server_config.crypto.
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(u8::MAX.into());

    // transport_config.
    // transport_config.max_concurrent_uni_streams(u16::)
    // transport_config.keep_alive_interval(Some(Duration::from_millis(300)));
    // transport_config.max_idle_timeout(Some(IdleTimeout::try_from(Duration::from_secs(5))?));
    // transport_config.max_idle_timeout(None);
    let endpoint = quic::Endpoint::server(server_config, addr)?;

    // let client_config;
    Ok(endpoint)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Message {
    /// 测试连通性
    Hello,

    /// 请求在服务器挂起等待
    Wait(String),

    /// 请求呼叫
    Call(String),

    /// 请求服务器可被呼叫用户列表
    QueryUsers,

    /// 请求结束通话
    Close,

    /// 服务器回应请求结果
    Response(Res),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Res {
    Ok,
    Err,
    Wait,
    Wake,
    UserList(Vec<String>),
}

impl Message {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        let json = serde_json::to_vec(&self).unwrap();
        json
    }
}

use std::fmt::Display;
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Message::Wait(name) => format!("等待被呼叫 name({})", name),
            Message::Call(name) => format!("呼叫 name({})", name),
            Message::QueryUsers => "查询等待列表".into(),
            Message::Close => "关闭通信".into(),
            Message::Response(_) => "Result".into(),
            Message::Hello => "Hello".into(),
        };
        f.write_str(&str)
    }
}

impl Res {
    pub fn is_ok(&self) -> bool {
        match self {
            Res::Ok => true,
            _ => false,
        }
    }
}
