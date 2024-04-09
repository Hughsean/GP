use opencv::{highgui, prelude::*, videoio};

fn capture_video() {
    // 0 是默认摄像头
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

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
            highgui::imshow("Video", &frame).unwrap();
        }

        let key = highgui::wait_key(10).unwrap();
        if key == 27 {
            break;
        }
        if n == 0 {
            println!("{}", frame.data_bytes().unwrap().len());
        }
        n += 1;
        if n == 100 {
            break;
        }
    }
    println!("{}", (std::time::Instant::now() - start).as_secs_f64())
}

fn main() {
    capture_video();
}
