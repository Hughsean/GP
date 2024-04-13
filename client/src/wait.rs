use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Context, Result};
use common::Message;

use cpal::traits::StreamTrait;
use log::debug;
use quic::{Connection, Endpoint, RecvStream};

use crate::audio::{make_input_stream, make_output_stream, vf32_to_vu8, vu8_to_vf32};

//
pub async fn wait(
    ctrl_endp: Endpoint,
    data_endp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    let ctrl_conn = ctrl_endp.connect(ctrl_addr, server_name)?;
    let ctrl_conn = ctrl_conn.await?;
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;

    let msg = Message::Wait(name.into());
    let msg = serde_json::to_string(&msg).unwrap();

    // 发送第一个请求
    send.write_all(msg.as_bytes()).await?;
    send.finish().await?;

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    if let Message::Result(info) = result {
        if info.is_ok() {
            let (_, waker) = ctrl_conn.open_bi().await?;
            let a_conn = data_endp.connect(data_addr, server_name)?.await?;
            let v_conn = data_endp.connect(data_addr, server_name)?.await?;
            waitcall(waker, a_conn, v_conn).await?;
        } else {
            return Err(anyhow!("请求错误"));
        }
    }

    Ok(ctrl_conn)
}

async fn waitcall(
    mut waker: RecvStream,
    a_conn: Connection,
    v_conn: Connection,
) -> anyhow::Result<()> {
    // 音频
    debug!("wait call");
    let recv = waker.read_to_end(usize::MAX).await?;

    let msg: common::Message = serde_json::from_slice(&recv).context("信息解析错误")?;
    if let common::Message::Result(common::Info::Wake) = msg {
        // 音频
        let fut1 = async {
            let (input_send, input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
            let (output_send, output_recv) = std::sync::mpsc::channel::<Vec<f32>>();
            let input_recv = Arc::new(tokio::sync::Mutex::new(input_recv));
            let output_send = Arc::new(tokio::sync::Mutex::new(output_send));

            // 音频
            let input_stream = make_input_stream(input_send);
            let output_stream = make_output_stream(output_recv);
            // 启动设备
            input_stream.play().unwrap();
            output_stream.play().unwrap();

            loop {
                let input_recv_c = input_recv.clone();
                let output_send_c = output_send.clone();

                if let Ok((mut send, mut recv)) = a_conn.open_bi().await {
                    // input send
                    let f1 = tokio::spawn(async move {
                        let data = input_recv_c.lock().await.recv()?;
                        let vu8 = vf32_to_vu8(data);
                        send.write_all(&vu8).await?;
                        send.finish().await?;

                        Ok::<(), anyhow::Error>(())
                    });
                    // output recv
                    let f2 = tokio::spawn(async move {
                        let data = recv.read_to_end(usize::MAX).await?;
                        let vf32 = vu8_to_vf32(data);
                        output_send_c.lock().await.send(vf32)?;

                        Ok::<(), anyhow::Error>(())
                    });

                    if let (Ok(r1), Ok(r2)) = tokio::join!(f1, f2) {
                        if r1.is_err() || r2.is_err() {
                            
                            break;
                        }
                    };
                } else {
                    break;
                }
            }
            // Ok::<(), anyhow::Error>(());
        };
    }

    Ok(())
}
