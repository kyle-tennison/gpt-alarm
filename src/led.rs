use std::sync::{Arc, mpsc::Receiver};

use std::sync::mpsc::{self, Sender, TryRecvError};
use std::time::Duration;

use crate::gpio::GPIOUtil;

const FLASH_PER_S: u8 = 5;
const SLEEP_INTERVAL: std::time::Duration = Duration::from_millis(1000 / FLASH_PER_S as u64);

#[derive(Debug)]
pub enum LEDState {
    Sleep,
    Nominal,
    Wait,
    Startup,
}

impl LEDState {
    fn num_flashes(&self) -> u8 {
        match &self {
            Self::Sleep => 1,
            Self::Nominal => 2,
            Self::Wait => 3,
            Self::Startup => 4,
        }
    }
}

pub struct LEDUtil {
    tx: Sender<LEDState>,
}

impl LEDUtil {
    pub fn new(gpio_util: Arc<GPIOUtil>) -> Self {
        let (tx, rx) = mpsc::channel::<LEDState>();

        let gpio_ref = gpio_util.clone();

        tokio::spawn(async {
            Self::async_worker(rx, gpio_ref).await;
        });

        LEDUtil { tx }
    }

    async fn async_worker(rx: Receiver<LEDState>, gpio_ref: Arc<GPIOUtil>) {
        println!("info: led task started");
        let mut state = LEDState::Startup;

        loop {
            match rx.try_recv() {
                Ok(update) => {
                    state = update;
                    println!("debug: updated led state to {:?}", state);
                    continue; // loop until empty
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            };

            let num_flash = state.num_flashes();
            let num_dormant = FLASH_PER_S - num_flash;

            for _ in 0..num_flash {
                gpio_ref.set_led(true);
                tokio::time::sleep(SLEEP_INTERVAL / 2).await;
                gpio_ref.set_led(false);
                tokio::time::sleep(SLEEP_INTERVAL / 2).await;
            }

            tokio::time::sleep(SLEEP_INTERVAL * num_dormant as u32).await;
        }
    }

    pub fn set_state(&self, state: LEDState) {
        self.tx.send(state).unwrap();
    }
}
