use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};

// pub const INPUT_SIZE: usize = 960 * std::mem::size_of::<f32>();

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
            move |data: &[f32], _info| send.send(data.to_vec()).unwrap(),
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
            move |data: &mut [f32], _info| {
                //这里怎么写
                // tokio::join!(async{});

                match recv.recv() {
                    Ok(recv_data) => data[0..recv_data.len()].copy_from_slice(&recv_data),
                    Err(e) => eprintln!("{}", e.to_string()),
                }
            },
            |err| eprintln!("Error during stream: {:?}", err),
            None,
        )
        .unwrap();
    stream
}

fn audio_send(send: quic::SendStream) {
    loop {}
}

fn audio_recv(recv: quic::RecvStream) {}

#[test]
fn m() {
    let (sendin, recvin) = std::sync::mpsc::channel::<Vec<f32>>();
    let s = make_input_stream(sendin);
    s.play().unwrap();
    std::thread::sleep(Duration::from_secs(8));
}
