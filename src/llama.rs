use core::time;
use serde_json::{Value, json};
use std::{
    thread,
    time::{Duration, Instant},
};
use subprocess::{Exec, Job, Redirection};

const LLAMA_SERVER_BIN: &str = "/opt/llama.cpp/build/bin/llama-server"; // update by machine
const HF_MODEL: &str = "LiquidAI/LFM2.5-VL-450M-GGUF";
const LLAMA_PORT: usize = 8080;
const LLAMA_HOST: &str = "127.0.0.1";
const LLAMA_TTL: u64 = 120; // timeout for llama
const NUM_PARALLEL: usize = 3;

// Will look through probability, return 'True' if the answer is True/Yes and false if the answer is `False/No`
pub async fn multimodal_bool_completion(
    text_prompt: &str,
    base64_image: &str,
    n_pred: usize,
) -> bool {
    let payload = json!({
        "max_tokens": n_pred,
        "n": NUM_PARALLEL,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpeg;base64,{base64_image}")
                        }
                    },
                    {
                        "type": "text",
                        "text": text_prompt
                    }
                ]
            }
        ]
    })
    .to_string();

    println!(
        "rust: sending payload of length {}",
        payload.to_string().len()
    );

    let response_str = (async {
        let client = reqwest::Client::new();
        let text = client
            .post(format!(
                "http://{LLAMA_HOST}:{LLAMA_PORT}/v1/chat/completions"
            ))
            .header("ContentType", "application/json")
            .body(payload)
            .send()
            .await?
            .text()
            .await?;
        Ok::<String, reqwest::Error>(text)
    })
    .await
    .expect("llama invocation failed");

    let json: Value = serde_json::from_str(&response_str).expect("invalid json from llama");
    // println!("rust: raw response {:#}", json);

    let mut true_prob: f64 = 0.;
    let mut false_prob: f64 = 0.;
    for item in json["choices"].as_array().unwrap() {
        let prob = 1.;
        let agent_resp = item["message"]["content"].as_str().unwrap();

        println!("the repsone is '{agent_resp}'");

        let token = agent_resp
            .split_ascii_whitespace()
            .next()
            .unwrap_or("Empty")
            .to_lowercase()
            .replace(".", "")
            .replace(" ", "")
            .replace("!", "")
            .replace(",", "")
            .replace(";", "");
        println!("the token is '{token}'");

        match token.as_str() {
            "true" | "yes" | "T" | "t" => true_prob += prob,
            "false" | "no" | "F" | "f" => false_prob += prob,
            _ => {}
        }
    }

    let response = true_prob > false_prob;
    println!(
        "rust: llama responded with {response}. The probability was T={} vs. F={}",
        true_prob, false_prob
    );
    response
}

pub async fn is_healthy() -> bool {
    // this is some bs
    let json = (async {
        let text = reqwest::get(format!("http://{LLAMA_HOST}:{LLAMA_PORT}/health"))
            .await?
            .text()
            .await?;

        println!("rust: response: {text}");

        let json: Value = serde_json::from_str(&text).expect("invalid json from llama");
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

    let exec = std::env::var("LLAMA_SERVER_BIN").unwrap_or(LLAMA_SERVER_BIN.to_string());

    let llama_log = std::fs::File::create("/tmp/gpt-alarm-llama.log").unwrap();

    println!("rut: using the executable at {exec}");
    let handle = thread::spawn(|| {
        Exec::cmd(exec)
            .arg("-hf")
            .arg(HF_MODEL)
            .arg("--host")
            .arg(LLAMA_HOST)
            .arg("--port")
            .arg(LLAMA_PORT.to_string())
            .arg("--n-gpu-layers")
            .arg("-1")
            .arg("--parallel")
            .arg(NUM_PARALLEL.to_string())
            .stdout(Redirection::File(llama_log))
            .stderr(Redirection::Merge)
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
