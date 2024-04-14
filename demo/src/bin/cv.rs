use opencv::{highgui, prelude::*, videoio};

fn capture_video() {
    // 0 是默认摄像头
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    // let w = cam.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap();
    // let h = cam.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap();

    // println!("{}", w * h);

    // println!("fps {}", cam.get(opencv::videoio::CAP_PROP_FPS).unwrap());
    // cam.set(opencv::videoio::CAP_PROP_FPS, 15f64).unwrap();
    // cam.set(videoio::CAP_PROP_FRAME_WIDTH, 300f64).unwrap();
    // cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 200f64).unwrap();

    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();
    if !opened {
        println!("Error: something wrong");
        return;
    }
    let _window = highgui::named_window("Video", highgui::WINDOW_AUTOSIZE).unwrap();
    let mut n = 0;

    let start = std::time::Instant::now();

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame).unwrap();

        if frame.size().unwrap().width > 0 {
            println!("{}", frame.data_bytes().unwrap().len());
            let params = opencv::types::VectorOfi32::new();
            let mut buf = opencv::types::VectorOfu8::new();
            opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &params).unwrap();
            let t = opencv::imgcodecs::imdecode(&buf, opencv::imgcodecs::IMREAD_COLOR).unwrap();
            assert_eq!(t.data_bytes().unwrap().len(),frame.data_bytes().unwrap().len());
            println!("{}", buf.len());
            highgui::imshow("Video", &t).unwrap();
        }

        let key = highgui::wait_key(10).unwrap();
        if key == 27 {
            break;
        }
    }
    println!("{}", (std::time::Instant::now() - start).as_secs_f64())
}

fn main() {
    capture_video();
}
