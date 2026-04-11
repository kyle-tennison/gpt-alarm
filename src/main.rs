use core::time;
use std::{process::Stdio, thread};
use subprocess::{Exec, Job, Redirection};
use tokio;

mod llama {
    use core::time;
    use reqwest;
    use serde_json::Value;
    use std::{process::Stdio, thread, time::{Duration, Instant}};
    use subprocess::{Exec, Job, Redirection};

    const LLAMA_SERVER_BIN: &str = "/opt/homebrew/bin/llama-server"; // update by machine
    const HF_MODEL: &str = "ggml-org/SmolVLM2-256M-Video-Instruct-GGUF";
    const LLAMA_PORT: usize = 8080;
    const LLAMA_HOST: &str = "127.0.0.1";
    const LLAMA_TTL: u64 = 60; // timeout for llama 

    pub async fn is_healthy() -> bool {
        // this is some bs
        let json = (async {
            let text = reqwest::get(format!("http://{LLAMA_HOST}:{LLAMA_PORT}/health"))
                .await?
                .text()
                .await?;

            println!("rust: response: {text}");

            let json: Value = serde_json::from_str(&text).unwrap();
            Ok::<Value, reqwest::Error>(json)
        })
        .await;

        if let Ok(json) = json {
            json["status"] == "ok"
        } else {
            false
        }
    }

    pub async fn start_server() -> Job {
        // spin up a llama server
        println!("rust: starting server in new thread");
        let handle = thread::spawn(|| {
            Exec::cmd(LLAMA_SERVER_BIN)
                .arg("-hf")
                .arg(HF_MODEL)
                .arg("--host")
                .arg(LLAMA_HOST)
                .arg("--port")
                .arg(LLAMA_PORT.to_string())
                .stdout(Redirection::None)
                .stderr(Redirection::None)
                .start()
                .expect("llama crashed on start")
        });
        let start_time = Instant::now();

        // wait for it to go online
        while (Instant::now() - start_time) < Duration::from_secs(LLAMA_TTL) {
            println!("rust: pinging llama...");

            if is_healthy().await {
                println!("rust: llama is running!");
                break;
            }
            tokio::time::sleep(time::Duration::from_secs(1)).await;
        }

        if (Instant::now() - start_time) >= Duration::from_secs(LLAMA_TTL) {
            panic!("rust: llama timeout, killing")
        }

        handle.join().unwrap()
    }
}

#[tokio::main]
async fn main() {
    println!("Starting up llama.cpp");

    let _job = llama::start_server().await;
    println!("Done")
}
