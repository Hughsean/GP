use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use opencv::{
    highgui,
    prelude::*,
    videoio::{self, VideoCapture},
};
use tracing::{debug, error, info};

const DELAY: u16 = 50;

pub fn display_c(recv: std::sync::mpsc::Receiver<Vec<u8>>) -> anyhow::Result<()> {
    loop {
        match recv.recv() {
            Ok(data) => {
                let buf = opencv::types::VectorOfu8::from(data);

                let frame = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR)
                    .inspect_err(|e| error!("decode err {e}"))?;

                if frame.size().unwrap().width > 0 {
                    highgui::imshow("Video", &frame).inspect(|_| debug!("播放数据帧"))?;
                }

                let key = highgui::wait_key(DELAY as i32)?;

                if key == 27 {
                    break;
                }
            }
            Err(e) => return Err(anyhow!("{e}")),
        }
    }
    Ok(())
}

pub fn capture_c(
    cam: &mut VideoCapture,
    send: std::sync::mpsc::Sender<Vec<u8>>,
) -> anyhow::Result<()> {
    let mut frame = Mat::default();
    loop {
        cam.read(&mut frame).inspect_err(|e| error!("{e}"))?;
        if frame.size()?.width > 0 {
            let params = opencv::types::VectorOfi32::new();
            let mut buf = opencv::types::VectorOfu8::new();

            // 对图片编码
            opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params)
                .inspect_err(|e| error!("encode {e}"))?;
            send.send(buf.to_vec())?;
            // debug!("编码");
            std::thread::sleep(Duration::from_millis(DELAY as u64));
        }
    }
}


pub async fn video_chanel(
    v_conn: quic::Connection,
    input_recv: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<Vec<u8>>>>,
    output_send: Arc<tokio::sync::Mutex<std::sync::mpsc::Sender<Vec<u8>>>>,
) {
    let input_recv_c = input_recv.clone();
    let output_send_c = output_send.clone();

    // 发送音频
    let a_conn_c = v_conn.clone();
    let f1 = tokio::spawn(async move {
        let lock = input_recv_c.lock().await;
        loop {
            match lock.recv() {
                Ok(data) => {
                    if let Ok(mut send) = a_conn_c.open_uni().await {
                        if send.write_all(&data).await.is_err() || send.finish().await.is_err() {
                            break error!("send");
                        }
                    } else {
                        break error!("open");
                    }
                }
                Err(e) => break error!("{e}"),
            };
        }
        a_conn_c.close(0u8.into(), b"close");
        Ok::<(), anyhow::Error>(())
    });

    // 接收音频
    let f2 = tokio::spawn(async move {
        let lock = output_send_c.lock().await;
        loop {
            match v_conn.accept_uni().await {
                Ok(mut recv) => {
                    if let Ok(data) = recv.read_to_end(usize::MAX).await {
                        if let Err(e) = lock.send(data) {
                            break error!("{e}");
                        }
                    } else {
                        break error!("read");
                    }
                }
                Err(e) => break error!("{e}"),
            }
        }
        v_conn.close(0u8.into(), b"close");
        Ok::<(), anyhow::Error>(())
    });

    let _ = tokio::join!(f1, f2);
    info!("音频结束");
}

#[allow(dead_code)]
pub async fn video(v_conn: quic::Connection, mut cam: VideoCapture) {
    // 打开窗口
    opencv::highgui::named_window("Video", opencv::highgui::WINDOW_AUTOSIZE)
        .inspect_err(|e| error!("打开窗口失败 {e}"))
        .unwrap();

    // 发送视频
    let v_conn_c = v_conn.clone();

    let f1 = tokio::spawn(async move {
        let mut n = 0u32;
        debug!("f1");
        loop {
            match capture(&mut cam) {
                Ok(data) => {
                    if let Ok(mut send) = v_conn_c.open_uni().await {
                        if send.write_all(&data).await.is_err() || send.finish().await.is_err() {
                            break error!("send");
                        } else {
                            if n % 40 == 0 {
                                debug!("frame send")
                            }
                            n += 1;
                        }
                    } else {
                        break error!("open");
                    }
                }
                Err(e) => break error!("break {e}"),
            };
        }
        v_conn_c.close(0u8.into(), b"close");
        Ok::<(), anyhow::Error>(())
    });

    // 接收视频
    let f2 = tokio::spawn(async move {
        let mut n = 0u32;
        debug!("f2");
        loop {
            match v_conn.accept_uni().await {
                Ok(mut recv) => {
                    if let Ok(data) = recv.read_to_end(usize::MAX).await {
                        if n % 40 == 0 {
                            debug!("frame recv")
                        }
                        n += 1;
                        if display(data).inspect_err(|e| error!("{e}")).is_err() {
                            break;
                        };
                    } else {
                        break error!("read");
                    }
                }
                Err(e) => break error!("{e}"),
            }
        }
        v_conn.close(0u8.into(), b"close");
        Ok::<(), anyhow::Error>(())
    });

    let _ = tokio::join!(f1, f2);
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

pub fn display(data: Vec<u8>) -> anyhow::Result<()> {
    let buf = opencv::types::VectorOfu8::from(data);

    let frame = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR)
        .inspect_err(|e| error!("decode err {e}"))?;

    debug!("解码");
    if frame.size().unwrap().width > 0 {
        highgui::imshow("Video", &frame).inspect(|_| debug!("播放数据帧"))?;
        debug!("show")
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
        let params = opencv::types::VectorOfi32::new();
        let mut buf = opencv::types::VectorOfu8::new();

        // 对图片编码
        opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params)
            .inspect_err(|e| error!("encode {e}"))?;
        // debug!("编码");
        std::thread::sleep(Duration::from_millis(10));

        Ok(buf.to_vec())
    } else {
        Err(anyhow!("Frame size <= 0"))
    }
}

#[tokio::test]

async fn t() {
    let mut cam = make_cam().unwrap();

    let buf = std::sync::Arc::new(tokio::sync::Mutex::new(vec![0u8; 10]));
    let buf_c = buf.clone();

    let t1 = tokio::spawn(async move {
        loop {
            if let Ok(data) = capture(&mut cam) {
                let mut lock = buf_c.lock().await;
                lock.clear();
                *lock = data;
                drop(lock);
                println!("send")
            } else {
                break;
            };
            // tokio::time::sleep(Duration::from_millis(millis))
        }
    });
    let t2 = tokio::spawn(async move {
        opencv::highgui::named_window("Video", opencv::highgui::WINDOW_AUTOSIZE)
            .inspect_err(|e| error!("打开窗口失败 {e}"))
            .unwrap();
        loop {
            let data: Vec<u8> = buf.lock().await.clone();
            if display(data).is_err() {
                break;
            };
            println!("play");
            // let data = capture(&mut cam).unwrap();
            // *buf.lock().await = data;
            // tokio::time::sleep(Duration::from_millis(millis))
        }
    });
    let _ = t1.await;
    let _ = t2.await;
}
