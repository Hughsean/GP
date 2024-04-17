use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use cpal::Stream;
use opencv::videoio::VideoCapture;
use tracing::info;

use crate::{
    audio::{make_input_stream, make_output_stream},
    video::make_cam,
};

pub type TMutex<T>=tokio::sync::Mutex<T>;


pub struct Device {
    pub a_output_stream: Stream,
    pub a_input_stream: Stream,
    pub cam: VideoCapture,
    // pub v: (Sender<u8>, Receiver<u8>,Arc<TMutex<>>),
    // pub a: (Sender<f32>, Receiver<f32>),

}

pub fn config() -> anyhow::Result<()> {
    //--------------------------------------------
    let (a_input_send, a_input_recv) = std::sync::mpsc::channel::<Vec<f32>>();
    let (a_output_send, a_output_recv) = std::sync::mpsc::channel::<Vec<f32>>();

    let a_input_recv_a = Arc::new(tokio::sync::Mutex::new(a_input_recv));
    let a_output_send_a = Arc::new(tokio::sync::Mutex::new(a_output_send.clone()));

    let a_input_stream = make_input_stream(a_input_send.clone());
    let a_output_stream = make_output_stream(a_output_recv);
    info!("音频设备配置成功");
    //--------------------------------------------
    // 视频设备
    let mut cam = make_cam()?;
    info!("摄像头启动");
    let (vinput_send, vinput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let (voutput_send, voutput_recv) = std::sync::mpsc::channel::<Vec<u8>>();
    let vinput_recv_a = Arc::new(tokio::sync::Mutex::new(vinput_recv));
    let voutput_send_a = Arc::new(tokio::sync::Mutex::new(voutput_send.clone()));
    //--------------------------------------------

    Ok(())
}
