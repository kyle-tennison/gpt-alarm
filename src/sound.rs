use std::sync::mpsc::{self, TryRecvError};
use std::{sync::Arc, time::Duration};

use crate::gpio::GPIOUtil;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum AlarmState {
    Active,
    Count(u8),
    Error,
    Disarmed,
}

pub struct SoundUtil {
    tx: mpsc::Sender<AlarmState>,
    gpio_util: Arc<GPIOUtil>,
}

impl SoundUtil {
    pub fn new(gpio_util: Arc<GPIOUtil>) -> Self {
        let (tx, rx) = mpsc::channel::<AlarmState>();

        let gpio_ref = gpio_util.clone();

        tokio::spawn(async {
            Self::async_worker(rx, gpio_ref).await;
        });

        SoundUtil {
            tx,
            gpio_util: gpio_util,
        }
    }

    async fn async_worker(rx: mpsc::Receiver<AlarmState>, gpio_util: Arc<GPIOUtil>) {
        println!("info: sound task started");
        let mut alarm_state = AlarmState::Disarmed;

        loop {
            match rx.try_recv() {
                Ok(update) => {
                    if update != alarm_state {
                        alarm_state = update;
                        println!("debug: updated alarm state to {:?}", alarm_state);
                    }
                    continue; // loop until empty
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

    pub fn set_state(&self, state: AlarmState) {
        println!("debug: requesting sound state {state:?}");
        self.tx
            .send(state)
            .expect("unable to update SoundUtil state");
    }

    pub async fn testsound(&self) {
        println!("debug: playing testsound");
        self.set_state(AlarmState::Count(2));
        tokio::time::sleep(Duration::from_millis(1300)).await;
        println!("debug: playing complete");
    }

    pub fn signal_start(&self) {
        self.set_state(AlarmState::Count(2));
    }
}

impl Drop for SoundUtil {
    fn drop(&mut self) {
        self.gpio_util.set_buzzer(true);
        std::thread::sleep(Duration::from_millis(750));
        self.gpio_util.set_buzzer(false);
    }
}
