use std::time::Duration;

use anyhow::anyhow;
use opencv::{
    core::Size,
    highgui,
    imgproc::resize,
    prelude::*,
    videoio::{self, VideoCapture},
};
use tracing::{debug, error};
pub fn play(data: Vec<u8>) -> anyhow::Result<()> {
    let buf = opencv::types::VectorOfu8::from(data);

    let frame = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR)
        .inspect_err(|e| error!("decode err {e}"))?;

    // debug!("解码");
    if frame.size().unwrap().width > 0 {
        highgui::imshow("Video", &frame).inspect(|_| debug!("播放数据帧"))?;
        // debug!("show")
    }

    let key = highgui::wait_key(10)?;

    if key == 27 {
        Err(anyhow!("break"))
    } else {
        Ok(())
    }
}

pub fn capture(cam: &mut VideoCapture) -> anyhow::Result<Vec<u8>> {
    let mut frame = Mat::default();

    cam.read(&mut frame).inspect_err(|e| error!("{e}"))?;
    if frame.size()?.width > 0 {
        let mut new_frame = Mat::default();
        resize(
            &frame,
            &mut new_frame,
            Size::new(600, 400),
            0.0,
            0.0,
            opencv::imgproc::INTER_LINEAR,
        )
        .inspect_err(|e| error!("Resize error: {e}"))?;

        debug!("frame {}", new_frame.data_bytes().unwrap().len());
        
        let params = opencv::types::VectorOfi32::new();
        let mut buf = opencv::types::VectorOfu8::new();

        // 对图片编码
        opencv::imgcodecs::imencode(".jpeg", &new_frame, &mut buf, &params)
            .inspect_err(|e| error!("encode {e}"))?;
        // debug!("编码");

        debug!("jpeg  {}", buf.len());
        std::thread::sleep(Duration::from_millis(10));
        // debug!("buf   {}", buf.len());
        // let enc = zstd::encode_all(buf.to_vec().as_slice(), 20).unwrap();
        // debug!("enc   {}", enc.len());
        // let dec = zstd::decode_all(enc.as_slice()).unwrap();

        Ok(buf.to_vec())
    } else {
        Err(anyhow!("Frame size <= 0"))
    }
}
fn capture_video() {
    // 0 是默认摄像头

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();
    if !opened {
        println!("Error: something wrong");
        return;
    }
    // let _window =
    highgui::named_window("Video", highgui::WINDOW_AUTOSIZE).unwrap();

    let data = capture(&mut cam).unwrap();
    play(data).unwrap();

    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}

fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("cv=debug")
        .init();

    capture_video();
}
