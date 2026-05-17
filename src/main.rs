use std::{sync::Arc, thread, time::Duration};

use base64::Engine;
use tempdir::TempDir;
use tokio::runtime::Runtime;

use crate::camera::Camera;

mod camera;
mod llama;
mod sound;
mod gpio;


const PRELIM_LAUNCH_SCRIPT: &str = "launch.sh";
const PROMPT: &str = "Is there a person in the bed? This is not a trick question. Only respond yes if he is visible. Please respond yes or no.";


// main thread (mt)
fn main() {

    // force root
    let is_root = unsafe { libc::getuid() } == 0;
    assert!(is_root, "must be sudo for GPIO");

    // run preliminary launch script to configure os
    let script = std::env::var("PRELIM_LAUNCH_SCRIPT").unwrap_or(PRELIM_LAUNCH_SCRIPT.to_string());
    std::process::Command::new("bash").arg(script).spawn().expect("preliminary script failed");
    println!("info: ran launch script");

    // camera needs to run on the main thread
    // let mt_camera = Camera::start();

    // main program runs here
    let at = thread::spawn(|| {
        let rt = Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            at_prog().await;
        });
    });

    // mt_camera.run_forever(); 
    // eprintln!("error: camera loop exited");

    at.join().unwrap();
}

// auxillary thread program. everything besides camera IO runs here
async fn at_prog(){
    let gpio_util = gpio::GPIOUtil::build();

    
    let gpio_arc = Arc::new(gpio_util);
    let sound_util = sound::SoundUtil::new(gpio_arc.clone());

    // some startup testing
    // Camera::save_photo("starup.jpg").await;
    sound_util.testsound().await;

    return;
    println!("info: starting up llama.cpp");
    let _job = llama::start_server().await;
    vlm_loop(sound_util).await;

}

async fn vlm_loop(sound_util: sound::SoundUtil) {

    let mut running_hist: [f32;5] = [0.;5]; // starts as false

    loop {
        let frame: Vec<u8> = Camera::fetch_frame_bytes().await;
        let frame_base64 = base64::engine::general_purpose::STANDARD.encode(frame);

        let result = llama::multimodal_bool_completion(PROMPT, &frame_base64, 30).await;

        running_hist.rotate_right(1);
        running_hist[0] = result as u8 as f32;

        let running_avg = running_hist.iter().map(|f| *f).sum::<f32>() / (running_hist.len() as f32);

        println!("rust: this iter: {result}. \tAverage: {running_avg}");

        


    }

}
