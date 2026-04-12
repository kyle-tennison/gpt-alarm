use std::{fs::File, io::{Cursor, Write}, iter::Inspect, path::{Path, PathBuf}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use async_process::{Command, ExitStatus};
use image::{ImageBuffer, ImageReader, Rgb, ImageFormat};
use nokhwa::{self, pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution}};
use base64::{self, Engine};

pub struct Camera<'a> {
    workdir: &'a Path,
    latest_frame: Arc::<Mutex<Option<String>>>
}

impl<'a> Camera<'a> {
    pub fn new(workdir: &'a Path) -> Camera<'a> {

        Camera { workdir: workdir, latest_frame: Arc::new(Mutex::new(None))}
    }


    pub fn begin_stream(&self) {

        let latest_frame = self.latest_frame.clone().to_owned();
        let workdir = PathBuf::from(self.workdir);

        thread::spawn(move || {
            println!("rust: starting up camera");
            let mut ncam = nokhwa::Camera::new(
                CameraIndex::Index(0),
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution)
            ).expect("failed to build ncam camera");
            
            ncam.open_stream().unwrap();


            // take a few test frames
            let start_time = Instant::now();

            for _ in 1..10{
                ncam.frame().expect("frame capture failed");
            }
            let end_time = Instant::now();

            let delta = (end_time - start_time).as_secs_f32();
            println!("rust: captured frame in {delta} seconds.");


            loop {
                let frame = ncam.frame().expect("frame capture failed");
                
                // resize
                let raw_buf = frame.decode_image::<RgbFormat>().expect("failed to decode frame");
                let scaled_buf = image::imageops::resize(&raw_buf, 854, 480, image::imageops::FilterType::Gaussian);
                
                // write to png 
                let mut png_buf = Cursor::new(Vec::<u8>::new());
                scaled_buf.write_to(&mut png_buf, ImageFormat::Png).unwrap();
                let png_buf = png_buf.into_inner();
                
                let active_image_path = workdir.join("active.png");
                println!("rust: writing image to {:?}", &active_image_path);
                File::create(active_image_path).unwrap().write(&png_buf).unwrap();
                
                let enc_base64 = base64::engine::general_purpose::STANDARD.encode(png_buf);
                
                {
                    let mut frame_lock = latest_frame.lock().unwrap();
                    *frame_lock = Some(enc_base64);
                    drop(frame_lock);
                }
        }
            

        });
    }
        
    

    /// Takes an image, returns as base64 JSON 
    pub fn take_image(&mut self) -> String {

        let start_time = Instant::now();

        loop {

            let mut frame_lock = self.latest_frame.lock().unwrap();

            if let Some(base64_data) = frame_lock.take() {
                let delta = (Instant::now() - start_time).as_secs_f32();
                println!("rust: recorded saved in {delta}s");
                return base64_data;
            }
            else{
                println!("rust: waiting for inbound image");
                drop(frame_lock);
                thread::sleep(Duration::from_secs(1));
            }
        }
    } 

}