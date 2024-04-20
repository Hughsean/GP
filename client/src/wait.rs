use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Context, Result};

use common::Message;

use cpal::traits::StreamTrait;

use quic::{Connection, Endpoint};
use tracing::{debug, info};

use crate::{
    audio::{audio_uni, make_input_stream, make_output_stream},
    video::{self, make_cam},
};

//
pub async fn wait(
    endp: Endpoint,
    aendp: Endpoint,
    vendp: Endpoint,
    ctrl_addr: SocketAddr,
    data_addr: SocketAddr,
    server_name: &str,
    name: &str,
) -> Result<Connection> {
    // 音频设备
    let (a_input_send, a_input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (a_output_send, a_output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

    let a_input_recv_a = Arc::new(tokio::sync::Mutex::new(a_input_recv));
    let a_output_send_a = Arc::new(tokio::sync::Mutex::new(a_output_send.clone()));

    let input_stream = make_input_stream(a_input_send.clone());
    let output_stream = make_output_stream(a_output_recv);
    info!("音频设备配置成功");
    //////////////////////////////////////////////
    // 视频设备
    let mut cam = make_cam()?;
    info!("摄像头启动");
    let (vinput_send, vinput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let (voutput_send, voutput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let vinput_recv_a = Arc::new(tokio::sync::Mutex::new(vinput_recv));
    let voutput_send_a = Arc::new(tokio::sync::Mutex::new(voutput_send.clone()));
    /////////////////////////////////////////////

    let ctrl_conn = endp.connect(ctrl_addr, server_name)?.await?;
    let (mut send, mut recv) = ctrl_conn.open_bi().await?;

    debug!("建立连接");

    // 发送第一个请求
    let msg = Message::Wait(name.into());
    send.write_all(&msg.to_vec_u8()).await?;
    send.finish().await?;

    debug!("发送请求");

    let result = recv.read_to_end(usize::MAX).await?;
    let result: Message = serde_json::from_slice(&result)?;

    debug!("获取请求结果");

    if let Message::Result(info) = result {
        // 创建音视频连接
        if info.is_ok() {
            debug!("请求被接受");
            let a_conn = aendp.connect(data_addr, server_name)?.await?;
            let v_conn = vendp.connect(data_addr, server_name)?.await?;

            // 'test: {
            //     let mut n = 0;
            //     loop {
            //         let (_, mut r) = v_conn.open_bi().await?;
            //         r.read_to_end(usize::MAX).await?;
            //         info!("read");
            //         sleep(std::time::Duration::from_secs(1)).await;
            //         n += 1;
            //         if n == 10 {
            //             break;
            //         }
            //     }
            // }

            info!("已创建音视频连接");

            info!("等待被呼叫");
            loop {
                let (_, mut wake_recv) = ctrl_conn.accept_bi().await?;

                let data_recv = wake_recv.read_to_end(usize::MAX).await?;
                let msg: common::Message =
                    serde_json::from_slice(&data_recv).context("信息解析错误")?;
                match msg {
                    Message::Result(common::Info::Wait) => {
                        debug!("收到服务器等待保活信息");
                        continue;
                    }
                    Message::Result(common::Info::Wake) => break,
                    _ => {
                        return Err(anyhow!("错误信息"));
                    }
                }
            }

            info!("被服务器唤醒");
            input_stream.play().unwrap();
            output_stream.play().unwrap();
            info!("音频设备启动");

            let t1 = std::thread::spawn(move || {
                let _ = video::capture_c(&mut cam, vinput_send.clone());
            });
            let t2 = std::thread::spawn(move || {
                let _ = video::display_c(voutput_recv);
            });

            let t3 = tokio::spawn(audio_uni(
                a_conn.clone(),
                a_input_recv_a.clone(),
                a_output_send_a.clone(),
            ));

            let _ = tokio::spawn(crate::video::video_chanel(
                v_conn.clone(),
                vinput_recv_a,
                voutput_send_a,
            ))
            .await;
            let _ = t3.await;
            let _ = t1.join();
            let _ = t2.join();
        } else {
            return Err(anyhow!("请求错误"));
        }
    }

    Ok(ctrl_conn)
}

// async move {
//     // 打开窗口
//     opencv::highgui::named_window("Video", opencv::highgui::WINDOW_AUTOSIZE)
//         .inspect_err(|e| error!("打开窗口失败 {e}"))
//         .unwrap();
//     // 发送视频
//     let v_conn_c = v_conn.clone();

//     let f1 = tokio::spawn(async move {
//         loop {
//             match crate::video::capture(&mut cam) {
//                 Ok(data) => {
//                     if let Ok(mut send) = v_conn_c.open_uni().await {
//                         if send.write_all(&data).await.is_err()
//                             || send.finish().await.is_err()
//                         {
//                             break error!("send");
//                         } else {
//                             debug!("frame send")
//                         }
//                     } else {
//                         break error!("open");
//                     }
//                 }
//                 Err(e) => break error!("break {e}"),
//             };
//         }
//         v_conn_c.close(0u8.into(), b"close");
//         Ok::<(), anyhow::Error>(())
//     });

//     // 接收视频
//     let f2 = tokio::spawn(async move {
//         loop {
//             match v_conn.accept_uni().await {
//                 Ok(mut recv) => {
//                     if let Ok(data) = recv.read_to_end(usize::MAX).await {
//                         if crate::video::display(data)
//                             .inspect_err(|e| error!("{e}"))
//                             .is_err()
//                         {
//                             break;
//                         };
//                         debug!("call play");
//                     } else {
//                         break error!("read");
//                     }
//                 }
//                 Err(e) => break error!("{e}"),
//             }
//         }
//         v_conn.close(0u8.into(), b"close");
//         Ok::<(), anyhow::Error>(())
//     });

//     let _ = tokio::join!(f1, f2);
//     info!("视频已断线")
// }
// .await;
