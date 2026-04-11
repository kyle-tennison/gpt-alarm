use std::path::Path;

use tempdir::TempDir;

mod llama;
mod camera;

const PROMPT: &str ="Is there a person in this image? Please respond yes or no.";

#[tokio::main]
async fn main() {
    println!("Starting up llama.cpp");
    let job = llama::start_server().await;

    let tmp_dir = TempDir::new("gpt-alarm").unwrap().into_path();
    let mut camera = camera::Camera::new(&tmp_dir);

    camera.begin_stream();

    println!("rust: beginning main loop");
    
    loop {
        let frame = camera.take_image();
        let _ = llama::multimodal_bool_completion(PROMPT, &frame, 30).await;
        break;

        // break
    }

    job.kill().unwrap();
    println!("Done")
}
