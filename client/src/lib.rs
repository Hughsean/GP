use common::endpoint_config::{make_endpoint, EndpointType};
use cpal::Stream;
use std::{fs, net::SocketAddr, sync::Arc};

use call::call;
use clap::Parser;
use command::Cli;

use quic::Endpoint;
use tracing::{error, info};

use crate::{
    audio::{make_input_stream, make_output_stream},
    video::make_cam,
};

pub const DELAY: u16 = 60;

pub struct Audio {
    /// 输出流
    pub play: Arc<tokio::sync::Mutex<Stream>>,
    /// 输入流
    pub record: Arc<tokio::sync::Mutex<Stream>>,
    /// 用于网络传输
    pub net_send_a: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
    pub net_recv_a: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
}

pub struct Video {
    /// 摄像头
    pub cam: Arc<tokio::sync::Mutex<opencv::videoio::VideoCapture>>,
    // pub send: std::sync::mpsc::Sender<Vec<u8>>,
    // pub recv: std::sync::mpsc::Receiver<Vec<u8>>,
    pub send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<u8>>>>,
    pub recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<u8>>>>,
    /// 用于网络传输
    pub net_send_v: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<u8>>>>,
    pub net_recv_v: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<u8>>>>,
}

pub struct Client {
    /// 终止
    pub stop: Arc<tokio::sync::RwLock<bool>>,
    pub cam: Arc<tokio::sync::Mutex<opencv::videoio::VideoCapture>>,
    // pub a: Arc<Audio>,
    // pub v: Arc<Video>,
    // pub endp: quic::Endpoint,
}

impl Client {
    pub fn new() -> anyhow::Result<Self> {
        // let (record_send, record_recv) = std::sync::mpsc::channel::<Vec<f32>>();
        // let (play_send, play_recv) = std::sync::mpsc::channel::<Vec<f32>>();

        // let net_recv_a = Arc::new(tokio::sync::Mutex::new(record_recv));
        // let net_send_a = Arc::new(tokio::sync::Mutex::new(play_send));

        // // 录音
        // let record = make_input_stream(record_send);
        // // 播放
        // let play = make_output_stream(play_recv);

        // let a = Audio {
        //     play: Arc::new(tokio::sync::Mutex::new(play)),
        //     record: Arc::new(tokio::sync::Mutex::new(record)),

        //     net_send_a,
        //     net_recv_a,
        // };

        let cam = Arc::new(tokio::sync::Mutex::new(make_cam()?));

        // // 捕获
        // let (capture_send, capture_recv) = std::sync::mpsc::channel::<Vec<u8>>();
        // // 播放
        // let (play_send, play_recv) = std::sync::mpsc::channel::<Vec<u8>>();

        // let send = Arc::new(tokio::sync::Mutex::new(capture_send));
        // let recv = Arc::new(tokio::sync::Mutex::new(play_recv));
        // // 向外传输
        // let net_recv_v = Arc::new(tokio::sync::Mutex::new(capture_recv));
        // // 接收数据
        // let net_send_v = Arc::new(tokio::sync::Mutex::new(play_send.clone()));

        // let v = Video {
        //     cam,
        //     send,
        //     recv,
        //     net_send_v,
        //     net_recv_v,
        // };

        Ok(Self {
            stop: Arc::new(tokio::sync::RwLock::new(false)),
            // a: Arc::new(a),
            // v: Arc::new(v),
            cam,
        })
    }
}

#[allow(dead_code)]
async fn main_() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("client=debug")
        .init();

    let cli = command::Cli::parse();
    let ctrl_addr: SocketAddr = cli
        .clone()
        .addr
        .unwrap_or("172.19.43.60:12345".into())
        .parse()
        .unwrap();

    let data_addr = SocketAddr::new(ctrl_addr.ip(), ctrl_addr.port() + 1);

    info!("ctrl_addr {}", ctrl_addr);
    info!("data_addr {}", data_addr);

    let endp = match config(cli.clone()) {
        Ok(ept) => ept,
        Err(e) => {
            println!("err: {e} [{} {}]", file!(), line!());
            return;
        }
    };

    let aendp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap();
    let vendp = make_endpoint(EndpointType::Client("0.0.0.0:0".parse().unwrap())).unwrap();

    match cli.command {
        command::Commands::Wait => {
            let conn = match wait::wait(
                endp.clone(),
                aendp,
                vendp,
                ctrl_addr,
                data_addr,
                &cli.server.unwrap_or("localhost".into()),
                &cli.name,
            )
            .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    error!("错误: {} line{}", err.to_string(), line!());
                    return;
                }
            };
            //todo

            conn.close(0u8.into(), b"done");
            endp.wait_idle().await;
        }
        command::Commands::Call { name } => match call(
            endp,
            aendp,
            vendp,
            ctrl_addr,
            data_addr,
            &cli.server.unwrap_or("localhost".into()),
            &name,
        )
        .await
        {
            Ok(_) => {
                info!("结束通话")
            }
            Err(e) => error!("{}", e.to_string()),
        },
        command::Commands::Query => println!("query"),
    }
}

fn config(cli: Cli) -> anyhow::Result<Endpoint> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&rustls::Certificate(fs::read(
        &cli.cert.unwrap_or("cert/cert.der".into()),
    )?))?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let client_config = quic::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quic::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}

pub mod audio;
mod call;
mod command;
pub mod video;
mod wait;
