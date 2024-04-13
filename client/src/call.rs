use std::net::SocketAddr;

use anyhow::anyhow;

use cpal::traits::StreamTrait;
use quic::Endpoint;

use crate::audio::{make_input_stream, make_output_stream};

pub async fn call(
    ctrl_endp: Endpoint,
    data_endp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> anyhow::Result<()> {
    let conn = ctrl_endp.connect(ctrl_addr, server_name)?;
    let conn = conn.await?;

    let (mut send, mut recv) = conn.open_bi().await?;

    let msg = common::Message::Call(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    // 第一个请求
    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: common::Message = serde_json::from_slice(&result).unwrap();

    if let common::Message::Result(common::Info::Ok) = result {
        let a_conn = data_endp.connect(data_addr, server_name)?.await?;
        let v_conn=data_endp.connect(data_addr, server_name)?.await?;
        calling(conn).await?;
    } else {
        return Err(anyhow!("请求错误"));
    }

    Ok(())
}

async fn calling(conn: quic::Connection) -> anyhow::Result<()> {
    // 音频处理
    let fut1 = async {
        let (sendin, recvin) = std::sync::mpsc::channel::<Vec<f32>>();
        let (sendout, recvout) = std::sync::mpsc::channel::<Vec<f32>>();

        let output = make_output_stream(recvout);
        let input = make_input_stream(sendin);
        input.play().unwrap();
        output.play().unwrap();
        loop {
            let (mut send, secr) = conn.accept_bi().await.unwrap();
            let buff32 = recvin.recv().unwrap();

            let mut bufu8: Vec<u8> = Vec::with_capacity(buff32.len() * 4);
            bufu8.copy_from_slice(unsafe {
                core::slice::from_raw_parts(buff32.as_ptr() as *const u8, buff32.len() * 4)
            });

            // send.write_all(buf);
        }

        // let futin=async{

        // };
    };

    // 视频处理
    let fut2 = async {};

    Ok(())
}

#[test]

fn f() {
    let buff32 = vec![12.1f32; 3];
    let mut bufu8: Vec<u8> = Vec::with_capacity(buff32.len() * 4);
    unsafe { bufu8.set_len(buff32.len() * 4) };
    bufu8.copy_from_slice(unsafe {
        core::slice::from_raw_parts(buff32.as_ptr() as *const u8, buff32.len() * 4)
    });

    let mut v: Vec<f32> = Vec::with_capacity(buff32.len());
    unsafe { v.set_len(buff32.len()) };
    v.copy_from_slice(unsafe {
        core::slice::from_raw_parts(bufu8.as_ptr() as *const f32, buff32.len())
    });

    println!("{:?}", v);
}
