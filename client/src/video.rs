use anyhow::{anyhow, Ok};
use log::info;
use opencv::{
    highgui,
    prelude::*,
    videoio::{self, VideoCapture},
};

pub async fn video(v_conn: quic::Connection) {
    let t1 = tokio::spawn(capture(v_conn.clone()));
    let t2 = tokio::spawn(play(v_conn));
    let _ = tokio::join!(t1, t2);
}

fn make_cam() -> anyhow::Result<VideoCapture> {
    let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    info!(
        "摄像头 画面宽度: {}",
        cam.get(videoio::CAP_PROP_FRAME_WIDTH)?
    );
    info!(
        "摄像头 画面高度: {}",
        cam.get(videoio::CAP_PROP_FRAME_HEIGHT)?
    );
    info!("摄像头 FPS: {}", cam.get(opencv::videoio::CAP_PROP_FPS)?);

    if videoio::VideoCapture::is_opened(&cam)? {
        Ok(cam)
    } else {
        Err(anyhow!(""))
    }
}

async fn play(v_conn: quic::Connection) -> anyhow::Result<()> {
    let _window = highgui::named_window("Video", highgui::WINDOW_AUTOSIZE)?;
    loop {
        let mut recv = v_conn.accept_uni().await?;
        let buf = recv.read_to_end(usize::MAX).await?;

        let buf = opencv::types::VectorOfu8::from(buf);
        let frame = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR)?;
        if frame.size().unwrap().width > 0 {
            highgui::imshow("Video", &frame).unwrap();
        }
        let _key = highgui::wait_key(10).unwrap();
    }
}

async fn capture(v_conn: quic::Connection) -> anyhow::Result<()> {
    let mut cam = make_cam()?;
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame).unwrap();
        if frame.size().unwrap().width > 0 {
            let mut send = v_conn.open_uni().await?;
            // 对图片编码
            let params = opencv::types::VectorOfi32::new();
            let mut buf = opencv::types::VectorOfu8::new();
            opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params)?;

            send.write_all(buf.as_slice()).await?;
            send.finish().await?;
        }
    }
}
