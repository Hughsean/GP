use anyhow::anyhow;
use tracing::{debug, error, info};
use opencv::{
    highgui,
    prelude::*,
    videoio::{self, VideoCapture},
};

pub async fn video(v_conn: quic::Connection, cam: VideoCapture) {
    let t1 = tokio::spawn(capture(v_conn.clone(), cam));
    let t2 = tokio::spawn(play(v_conn.clone()));
    let _ = tokio::join!(t1, t2);
    info!("视频已断线")
}

pub fn make_cam() -> anyhow::Result<VideoCapture> {
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
    loop {
        match v_conn.accept_uni().await {
            Ok(mut recv) => {
                let buf = recv.read_to_end(usize::MAX).await?;
                let buf = opencv::types::VectorOfu8::from(buf);

                let frame = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR)?;

                if frame.size().unwrap().width > 0 {
                    highgui::imshow("Video", &frame).unwrap();
                }
                debug!("play");
                let _key = highgui::wait_key(10).unwrap();
            }
            Err(e) => break error!("play err: {e}"),
        }
    }
    Ok(())
}

async fn capture(v_conn: quic::Connection, mut cam: VideoCapture) -> anyhow::Result<()> {
    let mut frame = Mat::default();
    loop {
        cam.read(&mut frame).unwrap();
        if frame.size().unwrap().width > 0 {
            match v_conn.open_uni().await {
                Ok(mut send) => {
                    // 对图片编码
                    let params = opencv::types::VectorOfi32::new();
                    let mut buf = opencv::types::VectorOfu8::new();

                    opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params)?;

                    debug!("图片编码 前{} 后{}", frame.data_bytes()?.len(), buf.len());

                    send.write_all(buf.as_slice()).await?;
                    send.finish().await?;
                }
                Err(e) => break error!("capture err: {e}"),
            };
        }
    }
    Ok(())
}
