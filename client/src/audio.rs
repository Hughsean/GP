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

#[test]
fn v2v() {
    let f = vec![12.1, 1.1, 3.4];
    let u = vf32_to_vu8(f);
    let f = vu8_to_vf32(u);

    println!("{:?}", f);
}
