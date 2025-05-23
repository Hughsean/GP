use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use std::{
    sync::mpsc::{self, SyncSender},
    thread,
};

fn record(sender: SyncSender<Vec<f32>>) {
    // 获取默认主机
    let host = cpal::default_host();

    // 获取默认输入设备
    let device = host.default_input_device().unwrap();

    println!("Using input device: {}", device.name().unwrap());

    // 获取默认输入格式
    let config = device.default_input_config().unwrap();

    println!("Using default input format: {:?}", config);
    // 构建输入流配置
    let mut config = device.default_input_config().unwrap().config();
    config.sample_rate.0 = 48000;
    println!("{} {}", config.channels, config.sample_rate.0);

    // 构建并运行输入流
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _| {
                // 这里的 `data` 包含了捕获的音频数据
                // 你可以在这里处理数据，比如写入文件等
                println!("send: {}", data.len() * 4);

                sender.send(data.into()).unwrap();
            },
            move |err| {
                eprintln!("Error during stream: {:?}", err);
            },
            None,
        )
        .unwrap();

    println!("Successfully built input stream. Starting...");
    // 开始捕获音频数据
    stream.play().unwrap();

    // 在非示例环境中，你需要在合适的时机停止输入流
    // stream.pause()?;

    // 阻塞主线程，保持捕获（示例）。在真实场景中，你将需要更复杂的流程控制
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    stream.pause().unwrap();
}

fn main() {
    let (s, r) = mpsc::sync_channel::<Vec<_>>(1);
    let t = thread::spawn(move || record(s));

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config: StreamConfig = device.default_output_config().unwrap().into();
    println!("buffer {}", config.sample_rate.0);
    // let mut n = 0;
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _info| {
                //这里怎么写
                match r.recv() {
                    Ok(v) => {
                        // if n < 2 {
                        println!("revc {};   data:{}", v.len(), data.len());
                        data[0..v.len()].copy_from_slice(&v);
                        // }
                        // n += 1;
                    }
                    Err(_) => {
                        // bc.wait();
                    }
                }
            },
            |_err| todo!(),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    t.join().unwrap();
    stream.pause().unwrap();
}
