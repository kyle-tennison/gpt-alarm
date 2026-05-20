use std::future::AsyncDrop;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::gpio::GPIOUtil;

#[derive(PartialEq, Debug)]
pub enum AlarmState {
    Active,
    Count(u8),
    Error,
    Disarmed,
}

pub struct SoundUtil {
    tx: mpsc::Sender<AlarmState>,
}

impl SoundUtil {
    pub fn new(gpio_util: Arc<GPIOUtil>) -> Self {
        let (tx, rx) = mpsc::channel::<AlarmState>(1);

        let gpio_ref = gpio_util.clone();

        tokio::spawn(async {
            Self::async_worker(rx, gpio_ref).await;
        });

        SoundUtil { tx }
    }

    async fn async_worker(mut rx: mpsc::Receiver<AlarmState>, gpio_util: Arc<GPIOUtil>) {
        println!("info: sound task started");
        let mut alarm_state = AlarmState::Disarmed;

        loop {
            match rx.try_recv() {
                Ok(update) => {
                    alarm_state = update;
                    println!("debug: updated alarm state to {:?}", alarm_state);
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            };

            match alarm_state {
                AlarmState::Active => {
                    gpio_util.set_buzzer(true);
                    tokio::time::sleep(Duration::from_millis(125)).await;
                    gpio_util.set_buzzer(false);
                    tokio::time::sleep(Duration::from_millis(125)).await;

                    gpio_util.set_buzzer(true);
                    tokio::time::sleep(Duration::from_millis(125)).await;
                    gpio_util.set_buzzer(false);

                    tokio::time::sleep(Duration::from_millis(750)).await;
                }
                AlarmState::Count(n) => {
                    for _ in 0..n {
                        gpio_util.set_buzzer(true);
                        tokio::time::sleep(Duration::from_millis(125)).await;
                        gpio_util.set_buzzer(false);
                        tokio::time::sleep(Duration::from_millis(125)).await;
                    }
                    alarm_state = AlarmState::Disarmed;
                }
                AlarmState::Error => {
                    gpio_util.set_buzzer(true);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    gpio_util.set_buzzer(false);
                    break;
                }
                AlarmState::Disarmed => tokio::time::sleep(Duration::from_millis(500)).await,
            };
        }
        println!("warn: sound async worker exiting")
    }

    pub async fn set_state(&self, state: AlarmState) {
        self.tx
            .send(state)
            .await
            .expect("unable to update SoundUtil state");
    }

    pub async fn testsound(&self) {
        println!("debug: playing testsound");
        self.set_state(AlarmState::Count(2)).await;
        tokio::time::sleep(Duration::from_millis(1300)).await;
        println!("debug: playing complete");
    }

    pub async fn signal_start(&self) {
        self.set_state(AlarmState::Count(2)).await;
    }
}

impl Drop for SoundUtil {
    fn drop(&mut self) {
        println!("warn: sound util dropped outside of async");
    }
}

impl AsyncDrop for SoundUtil {
    async fn drop(self: std::pin::Pin<&mut Self>) {
        self.set_state(AlarmState::Error).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
