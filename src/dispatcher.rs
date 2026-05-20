use crate::{
    camera::FrameFetcher,
    gpio::GPIOUtil,
    led::{self, LEDUtil},
    llama,
    sound::{AlarmState, SoundUtil},
};
use base64::Engine;
use chrono::Timelike;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use subprocess::Job;

// VLM Query Parameters
const PROMPT: &str = "Is there a person *lying* in the bed? Respond one word: 'yes' or 'no'. \
                      This is *not* a trick question. Only say yes if he is clearly visible \
                      *and laying down.* **DO NOT say 'yes' if he is standing/sitting.**";

const CONFIDENCE_THRESHOLD: f32 = 0.80;

// Interval when alarm is active [HR, MIN] (military)
// TODO: midnight overlap support
const INTERVAL_BEGIN: [u32; 2] = [7, 00];
const INTERVAL_END: [u32; 2] = [12, 00];

#[allow(dead_code)]
pub struct Dispatcher {
    sound_util: SoundUtil,
    frame_fetcher: FrameFetcher,
    led_util: LEDUtil,
    llama_job: Job,
}

impl Dispatcher {
    pub async fn startup(frame_fetcher: FrameFetcher) -> Self {
        let gpio_util = Arc::new(GPIOUtil::build());
        let sound_util = SoundUtil::new(gpio_util.clone());
        let led_util = LEDUtil::new(gpio_util.clone());

        // some startup testing
        sound_util.testsound().await;
        _ = tokio::fs::create_dir("startup-samples").await;
        for i in 0..3 {
            frame_fetcher
                .save_photo(format!("startup-samples/starup-{i}.jpg").as_str())
                .await;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        println!("info: starting up llama.cpp");
        led_util.set_state(led::LEDState::Wait);
        let llama_job = llama::start_server().await;
        println!("info: llama setup finished");
        sound_util.signal_start();

        println!("info: startup success");
        Self {
            sound_util,
            frame_fetcher,
            led_util,
            llama_job,
        }
    }

    /// check if it's within the specified time
    fn is_active_time() -> bool {
        let now = chrono::Local::now();

        let hour = now.hour();
        let min = now.minute();

        println!(
            "info: checking time. alarm is active between {ahs:02}:{ams:02} - {ahe:02}:{ame:02}. it is currently {nh:02}:{nm:02}.",
            ahs = INTERVAL_BEGIN[0],
            ams = INTERVAL_BEGIN[1],
            ahe = INTERVAL_END[0],
            ame = INTERVAL_END[1],
            nh = hour,
            nm = min,
        );

        let begin_mins = INTERVAL_BEGIN[0] * 60 + INTERVAL_BEGIN[1];
        let end_mins = INTERVAL_END[0] * 60 + INTERVAL_END[1];
        let current_mins = hour * 60 + min;

        begin_mins <= current_mins && current_mins < end_mins
    }

    pub async fn run(&self) {
        println!("info: starting prog run");
        self.led_util.set_state(led::LEDState::Sleep);

        loop {
            while !Self::is_active_time() {
                tokio::time::sleep(Duration::from_mins(1)).await;
            }
            self.led_util.set_state(led::LEDState::Nominal);
            self.sound_util.set_state(AlarmState::Active);

            self.vlm_loop().await;
            println!("info: going back to sleep");
            self.led_util.set_state(led::LEDState::Sleep);
            self.sound_util.set_state(AlarmState::Disarmed);
        }
    }

    async fn vlm_loop(&self) {
        let mut running_hist: [f32; 5] = [CONFIDENCE_THRESHOLD; 5]; // starts in the middle

        let mut last_iter_timestamp = Instant::now();
        while Self::is_active_time() {
            let frame: Vec<u8> = self.frame_fetcher.fetch_frame_bytes();

            #[cfg(debug_assertions)]
            {
                self.frame_fetcher
                    .save_photo_bytes("debug_frame.jpg", &frame)
                    .await;
            }

            let frame_base64 = base64::engine::general_purpose::STANDARD.encode(frame);

            let result = llama::multimodal_bool_completion(PROMPT, &frame_base64, 30).await;

            running_hist.rotate_right(1);
            running_hist[0] = result as u8 as f32;

            let running_avg =
                running_hist.iter().copied().sum::<f32>() / (running_hist.len() as f32);

            if running_avg > CONFIDENCE_THRESHOLD {
                self.sound_util.set_state(AlarmState::Active);
            } else {
                self.sound_util.set_state(AlarmState::Disarmed);
            }
            tokio::task::yield_now().await;

            let this_iter_timestamp = Instant::now();
            let elapsed = (this_iter_timestamp - last_iter_timestamp).as_secs_f32();
            last_iter_timestamp = this_iter_timestamp;
            println!(
                "\n === this iter: {result} - running avg {running_avg:.2} - elapsed {elapsed:.3} s === \n "
            );
        }
    }
}
