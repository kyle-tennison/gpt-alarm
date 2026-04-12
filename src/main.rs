use std::{path::Path, time::Instant};

use tempdir::TempDir;

use crate::sound::SoundUtil;

mod llama;
mod camera;
mod sound;

const PROMPT: &str ="Is there a person in the bed? This is not a trick question. Only respond yes if he is visible. Please respond yes or no.";

#[tokio::main(flavor = "local")]
async fn main() {
    println!("Starting up llama.cpp");
    let job = llama::start_server().await;

    let tmp_dir = TempDir::new("gpt-alarm").unwrap().into_path();
    let mut camera = camera::Camera::new(&tmp_dir);
    let sound = SoundUtil::new();

    camera.begin_stream();
    

    println!("rust: beginning main loop");
    
    let mut running_hist: Vec<f32> = Vec::with_capacity(size_of::<f32>()*5);

    loop {
        let frame = camera.take_image();
        let result = llama::multimodal_bool_completion(PROMPT, &frame, 30).await;

        running_hist.insert(0, if result {1.} else {0.});

        if running_hist.len() > 6{
            running_hist.pop();
        }

        let running_avg = running_hist.iter().map(|f| *f).sum::<f32>() / (running_hist.len() as f32);

        println!("rust: this iter: {result}. \tAverage: {running_avg}");

        sound.update_state(running_avg > 0.5);        
    }

    job.kill().unwrap();
    println!("Done")
}
