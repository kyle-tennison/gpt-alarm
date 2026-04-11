use std::{fs::File, io::{Cursor, Write}, iter::Inspect, path::Path};
use async_process::{Command, ExitStatus};
use image::{ImageBuffer, ImageReader, Rgb, ImageFormat};
use tokio::time::Instant;
use nokhwa::{self, pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution}};
use base64::{self, Engine};

pub struct Camera<'a> {
    workdir: &'a Path,
    nokhwa: nokhwa::Camera
}

impl<'a> Camera<'a> {
    pub fn new(workdir: &'a Path) -> Camera<'a> {


        let mut nokhwa = nokhwa::Camera::new(
            CameraIndex::Index(0),
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution)
        ).expect("failed to build nokhwa camera");

        // nokhwa.set_resolution(
        //     Resolution::new(1280, 700)
        // ).unwrap();

        println!("camera contols: {:#?}", nokhwa.camera_controls());

        Camera { workdir: workdir, nokhwa }
    }


    pub fn begin_stream(&mut self) {
        println!("rust: starting up camera");
        self.nokhwa.open_stream().unwrap();

        // take a few test frames
        let start_time = Instant::now();

        for _ in 1..10{
            self.nokhwa.frame().expect("frame capture failed");
        }
        let end_time = Instant::now();

        let delta = (end_time - start_time).as_secs_f32();
        println!("rust: captured frame in {delta} seconds.");
        }

    /// Takes an image, returns as base64 JSON 
    pub fn take_image(&mut self) -> String {
        if !self.nokhwa.is_stream_open() {
            panic!("tried to take image with closed stream")
        }

        let start_time = Instant::now();
        let frame = self.nokhwa.frame().expect("frame capture failed");
        
        // resize
        let raw_buf = frame.decode_image::<RgbFormat>().expect("failed to decode frame");
        let scaled_buf = image::imageops::resize(&raw_buf, 854, 480, image::imageops::FilterType::Gaussian);

        // write to png 
        let mut png_buf = Cursor::new(Vec::<u8>::new());
        scaled_buf.write_to(&mut png_buf, ImageFormat::Png).unwrap();
        let png_buf = png_buf.into_inner();

        let active_image_path = self.workdir.join("active.png");
        println!("rust: writing image to {:?}", &active_image_path);
        File::create(active_image_path).unwrap().write(&png_buf).unwrap();

        let enc_base64 = base64::engine::general_purpose::STANDARD.encode(png_buf);


        let delta = (Instant::now() - start_time).as_secs_f32();
        println!("rust: recorded frame in {delta}s");

        enc_base64

        // convert to base64
        // let engine = base64::engine::general_purpose::STANDARD.encode(png_buf.into_inner());
    } 

}