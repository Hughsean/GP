use std::sync::Arc;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    StreamConfig,
};
use log::{debug, error, info, warn};

pub fn make_input_stream(send: std::sync::mpsc::Sender<Vec<f32>>) -> cpal::Stream {
    // 获取默认主机
    let host = cpal::default_host();
    // 获取默认输入设备
    let device = host.default_input_device().unwrap();
    // 获取默认输入格式
    let config = device.default_input_config().unwrap();
    // 构建输入流配置
    let config: StreamConfig = config.into();

    // 构建并运行输入流
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info| match send.send(data.to_vec()) {
                Ok(_) => (),
                Err(e) => error!("{}", e.to_string()),
            },
            |err| eprintln!("Error during stream: {:?}", err),
            None,
        )
        .unwrap();
    stream
}

pub fn make_output_stream(recv: std::sync::mpsc::Receiver<Vec<f32>>) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config: StreamConfig = device.default_output_config().unwrap().into();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _info| match recv.recv() {
                Ok(recv_data) => data[0..recv_data.len()].copy_from_slice(&recv_data),
                Err(e) => error!("{}", e.to_string()),
            },
            |err| eprintln!("Error during stream: {:?}", err),
            None,
        )
        .unwrap();
    stream
}

pub async fn audio(
    a_conn: quic::Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
) {
    if let Ok((mut send, mut recv)) = a_conn.open_bi().await {
        info!("流建立 {}", line!());
        let mut sbuf = vec![0u8; 5 * 1024];
        let mut rbuf = vec![0u8; 5 * 1024];

        let fut1 = async move {
            loop {
                match input_recv.lock().await.recv() {
                    Ok(data) => {
                        debug!("读取设备音频数据");
                        let data = vf32_to_vu8(data);
                        data_write_to_buf(&mut sbuf, data);
                        if send.write_all(&sbuf).await.is_err() {
                            break;
                        }
                        debug!("发送音频数据");
                    }
                    Err(e) => break error!("{e} {}", line!()),
                }
            }
        };

        let fut2 = async move {
            loop {
                match recv.read_exact(rbuf.as_mut_slice()).await {
                    Ok(_) => {
                        debug!("接收音频数据");
                        let data = data_read_from_buf(&rbuf);
                        let data = vu8_to_vf32(data);
                        match output_send.lock().await.send(data) {
                            Ok(()) => debug!("发送音频数据到设备"),
                            Err(e) => break error!("{e} {}", line!()),
                        }
                    }
                    Err(e) => break error!("{e} {}", line!()),
                }
            }
        };

        let t1 = tokio::spawn(fut1);
        let t2 = tokio::spawn(fut2);
        let _ = t1.await;
        let _ = t2.await;
    } else {
        error!("打开流失败 {}", line!());
    }
}

pub async fn audio__(
    a_conn: quic::Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
) {
    let input_recv_c = input_recv.clone();
    let output_send_c = output_send.clone();

    // 发送音频
    let a_conn_c = a_conn.clone();
    let f1 = tokio::spawn(async move {
        let lock = input_recv_c.lock().await;
        loop {
            match lock.recv() {
                Ok(data) => {
                    let vu8 = vf32_to_vu8(data);

                    if let Ok(mut send) = a_conn_c.open_uni().await {
                        if send.write_all(&vu8).await.is_err() || send.finish().await.is_err() {
                            break error!("{}", line!());
                        }
                    } else {
                        break error!("{}", line!());
                    }
                }
                Err(e) => break error!("{e} {}", line!()),
            };
        }
        Ok::<(), anyhow::Error>(())
    });

    // 接收音频
    let f2 = tokio::spawn(async move {
        let lock = output_send_c.lock().await;
        loop {
            match a_conn.accept_uni().await {
                Ok(mut recv) => {
                    if let Ok(data) = recv.read_to_end(usize::MAX).await {
                        let vf32 = vu8_to_vf32(data);
                        if let Err(e) = lock.send(vf32) {
                            break error!("{e} {}", line!());
                        }
                    } else {
                        break error!("{}", line!());
                    }
                }
                Err(e) => break error!("{e} {}", line!()),
            }
        }

        Ok::<(), anyhow::Error>(())
    });

    let _ = tokio::join!(f1, f2);

    // fut1
}
pub async fn audio_(
    a_conn: quic::Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<f32>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<f32>>>>,
) {
    loop {
        let input_recv_c = input_recv.clone();
        let output_send_c = output_send.clone();

        match a_conn.open_bi().await {
            Ok((mut send, mut recv)) => {
                //
                let fut1 = async move {
                    match input_recv_c.lock().await.recv() {
                        Ok(data) => {
                            let data = vf32_to_vu8(data);
                            if !(send.write_all(&data).await.is_ok() && send.finish().await.is_ok())
                            {
                                error!("发送错误 line{}", line!());
                            }
                        }
                        Err(e) => error!("{e} line{}", line!()),
                    }
                };

                let fut2 = async move {
                    match recv.read_to_end(usize::MAX).await {
                        Ok(data) => {
                            let data = vu8_to_vf32(data);
                            match output_send_c.lock().await.send(data) {
                                Ok(_) => (),
                                Err(e) => error!("{e} line{}", line!()),
                            }
                        }
                        Err(e) => error!("{e} line{}", line!()),
                    }
                };

                let t1 = tokio::spawn(fut1);
                let t2 = tokio::spawn(fut2);
                let _ = tokio::join!(t1, t2);
            }
            Err(e) => break error!("{e} line{}", line!()),
        }
    }

    info!("音频已断线")
}

pub fn vf32_to_vu8(vf32: Vec<f32>) -> Vec<u8> {
    let vu8len = vf32.len() * 4;

    let mut ret = Vec::with_capacity(vu8len);
    unsafe { ret.set_len(vu8len) };

    ret.copy_from_slice(unsafe { core::slice::from_raw_parts(vf32.as_ptr() as *const u8, vu8len) });

    ret
}

pub fn vu8_to_vf32(vu8: Vec<u8>) -> Vec<f32> {
    let vf32len = vu8.len() / 4;

    let mut ret = Vec::with_capacity(vf32len);
    unsafe { ret.set_len(vf32len) };

    ret.copy_from_slice(unsafe {
        core::slice::from_raw_parts(vu8.as_ptr() as *const f32, vf32len)
    });

    ret
}

fn data_write_to_buf(buf: &mut [u8], mut data: Vec<u8>) {
    let len = data.len() as u32;
    let len_bytes = unsafe { std::mem::transmute::<u32, [u8; 4]>(len) };
    let mut temp = Vec::from(&len_bytes);
    temp.append(&mut data);
    buf[..temp.len()].copy_from_slice(&temp);
}

fn data_read_from_buf(buf: &[u8]) -> Vec<u8> {
    let mut len = [0u8; 4];
    len.copy_from_slice(&buf[..4]);
    let len = unsafe { std::mem::transmute::<[u8; 4], u32>(len) };
    let len = len as usize;
    buf[4..len + 4].to_vec()
}

#[test]
fn v2v() {
    let f = vec![12.1, 1.1, 3.4];
    let u = vf32_to_vu8(f);
    let f = vu8_to_vf32(u);

    println!("{:?}", f);
}

#[test]
fn vec() {
    let data = vec![12u8, 3, 4, 5, 12, 45];

    let mut buf = [0u8; 1000];

    data_write_to_buf(&mut buf, data);

    let data = data_read_from_buf(&buf);
    println!("{data:?}")
}
