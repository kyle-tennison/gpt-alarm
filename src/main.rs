use std::{thread, time::Duration};

use base64::Engine;
use tempdir::TempDir;
use tokio::runtime::Runtime;

use crate::camera::Camera;

mod camera;
mod llama;
mod sound;

use gpio::{GpioIn, GpioOut};

const PROMPT: &str = "Is there a person in the bed? This is not a trick question. Only respond yes if he is visible. Please respond yes or no.";

fn main() {
    let mut gpio9 = gpio::sysfs::SysFsGpioOutput::open(9).unwrap();
    gpio9.set_high();
    thread::sleep(Duration::from_secs(1));
    gpio9.set_low();

}

// main thread (mt)
fn main_2() {

    // camera needs to run on the main thread
    let mt_camera = Camera::build();

    // main program runs here
    let at = thread::spawn(|| {
        let rt = Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            at_prog().await;
        });
    });

    mt_camera.run_forever(); 
    eprintln!("error: camera loop exited");

    at.join().unwrap();
}

// auxillary thread program. everything besides camera IO runs here
async fn at_prog(){

    println!("info: starting up llama.cpp");
    let _job = llama::start_server().await;
    vlm_loop().await;

}

async fn vlm_loop() {

    let mut running_hist: [f32;5] = [0.;5]; // starts as false

    loop {
        let frame: Vec<u8> = Camera::fetch_frame_bytes();
        let frame_base64 = base64::engine::general_purpose::STANDARD.encode(frame);

        let result = llama::multimodal_bool_completion(PROMPT, &frame_base64, 30).await;

        running_hist.rotate_right(1);
        running_hist[0] = result as u8 as f32;

        let running_avg = running_hist.iter().map(|f| *f).sum::<f32>() / (running_hist.len() as f32);

        println!("rust: this iter: {result}. \tAverage: {running_avg}");

    }

}
