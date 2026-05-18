use std::{sync::Arc, thread, time::Duration};

use base64::Engine;
use tokio::{runtime::Runtime, time::Instant};

use crate::camera::{CamService, FrameFetcher};

mod camera;
mod gpio;
mod led;
mod llama;
mod sound;

const PROMPT: &str = "Is there a person in the bed? This is not a trick question. Only respond yes if he is visible. Please respond yes or no.";
const CONFIDENCE_THRESHOLD: f32 = 0.5;
const PRELIM_LAUNCH_SCRIPT: &str = "launch.sh";

// main thread (mt)
fn main() {
    // force root
    let is_root = unsafe { libc::getuid() } == 0;
    assert!(is_root, "must be sudo for GPIO");

    // run preliminary launch script to configure os
    let script = std::env::var("PRELIM_LAUNCH_SCRIPT").unwrap_or(PRELIM_LAUNCH_SCRIPT.to_string());
    std::process::Command::new("bash")
        .arg(script)
        .spawn()
        .expect("preliminary script failed on start")
        .wait()
        .expect("preliminary process failed");
    println!("info: ran launch script");

    // camera servie needs to run on the main thread
    let mut cam_service = CamService::start();
    let frame_fetcher = cam_service.get_fetcher().unwrap();

    // main program runs here
    let at = thread::spawn(move || {
        let rt = Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            at_prog(
                frame_fetcher,
            ).await;
        });
    });

    cam_service.run_forever();
    eprintln!("error: camera loop exited");

    at.join().unwrap();
}

// auxillary thread program. everything besides camera IO runs here
async fn at_prog(
    frame_fetcher: FrameFetcher
) {
    println!("info: start at_prog");
    let gpio_util = Arc::new(gpio::GPIOUtil::build());
    let sound_util = sound::SoundUtil::new(gpio_util.clone());

    // some startup testing
    _ = tokio::fs::create_dir("startup-samples").await;
    for i in 0..3 {
        frame_fetcher.save_photo(format!("startup-samples/starup-{i}.jpg").as_str()).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    
    gpio_util.set_led(true);
    println!("info: starting up llama.cpp");
    let _job = llama::start_server().await;
    gpio_util.set_led(false);

    sound_util.testsound().await;

    vlm_loop(sound_util, frame_fetcher).await;
}

async fn vlm_loop(sound_util: sound::SoundUtil, frame_fetcher: FrameFetcher) {
    let mut running_hist: [f32; 5] = [0.; 5]; // starts as false

    let mut last_iter_timestamp = Instant::now();
    loop {
        let frame: Vec<u8> = frame_fetcher.fetch_frame_bytes();

        frame_fetcher.save_photo_bytes("debug_frame.jpg", &frame).await;

        let frame_base64 = base64::engine::general_purpose::STANDARD.encode(frame);

        let result = llama::multimodal_bool_completion(PROMPT, &frame_base64, 30).await;

        running_hist.rotate_right(1);
        running_hist[0] = result as u8 as f32;

        let running_avg = running_hist.iter().copied().sum::<f32>() / (running_hist.len() as f32);

        
        if running_avg > CONFIDENCE_THRESHOLD {
            sound_util.set_state(sound::AlarmState::Active).await;
        } else {
            sound_util.set_state(sound::AlarmState::Disarmed).await;
        }

        let this_iter_timestamp = Instant::now();
        let elapsed = (this_iter_timestamp - last_iter_timestamp).as_secs_f32();
        last_iter_timestamp = this_iter_timestamp;
        println!("\n === this iter: {result} - running avg {running_avg:.2} - elapsed {elapsed:.3} s === \n ");
    }
}
