use base64::{self, Engine};
use image::{ImageFormat};
use nokhwa::{
    self,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
};
use std::{
    collections::VecDeque, io::Cursor, path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant}
};

pub struct Camera<'a> {
    _workdir: &'a Path,
    frame_queue: Arc<Mutex<VecDeque<String>>>,
}

impl<'a> Camera<'a> {
    pub fn new(workdir: &'a Path) -> Camera<'a> {
        Camera {
            _workdir: workdir,
            frame_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn begin_stream(&self) {
        let frame_queue = self.frame_queue.clone().to_owned();

        thread::spawn(move || {
            println!("rust: starting up camera");
            let mut ncam = nokhwa::Camera::new(
                CameraIndex::Index(0),
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
            )
            .expect("failed to build ncam camera");

            ncam.open_stream().unwrap();

            // Uncommment the line below to see what resolutions are natively compatible
            // we wanna set this here bc its expensive to resize manually
            // println!("{:#?}", ncam.compatible_list_by_resolution(nokhwa::utils::FrameFormat::RAWRGB));
            ncam.set_resolution(Resolution {
                width_x: 640,
                height_y: 480,
            })
            .unwrap();

            // take a few test frames to let exposure settle
            for _ in 1..10 {
                ncam.frame().expect("frame capture failed");
            }

            'forev: loop {

                let guard = frame_queue.lock().unwrap();
                if guard.len() > 2 {
                    drop(guard);
                    thread::sleep(Duration::from_millis(100));
                    continue 'forev;
                }
                drop(guard);

                println!("rust: capturing new frame");
                let start = Instant::now();
                let frame = ncam.frame().expect("frame capture failed");

                let raw_buf = frame
                    .decode_image::<RgbFormat>()
                    .expect("failed to decode frame");

                // write to png
                let mut png_buf = Cursor::new(Vec::<u8>::new());
                raw_buf.write_to(&mut png_buf, ImageFormat::Png).unwrap();
                let png_buf = png_buf.into_inner();

                let enc_base64 = base64::engine::general_purpose::STANDARD.encode(png_buf);

                println!("Captured frame in {}s", (Instant::now()-start).as_secs_f32());

                let mut guard = frame_queue.lock().unwrap();
                guard.push_front(enc_base64);
                drop(guard)
            }
        });
    }

    /// Takes an image, returns as base64 JSON
    pub fn take_image(&mut self) -> String {
        let start_time = Instant::now();

        loop {
            let mut guard = self.frame_queue.lock().unwrap();
            let value = guard.pop_back();
            drop(guard);

            if let Some(base64_data) = value {
                let delta = (Instant::now() - start_time).as_secs_f32();
                println!("rust: recorded saved in {delta}s");
                return base64_data;
            } else {
                println!("rust: waiting for inbound image");
                println!("\x07");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
