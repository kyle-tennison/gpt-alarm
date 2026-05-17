use std::time::Duration;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use tokio::sync::mpsc::{self, error::TryRecvError};

const BUZZ_GPIO_OFFSET: u32 = 144;
const LIGHT_GPIO_OFFSET: u32 = 85;
const CHIP_PATH: &str = "/dev/gpiochip0"; // same for both

struct GPIOUtil {
    buzzer_handle: LineHandle,
    led_handle: LineHandle,
}

impl GPIOUtil {

    pub fn build() -> Self {

        let mut chip = Chip::new(CHIP_PATH).expect("failed to access gpiochip");

        let buzz_line = chip.get_line(BUZZ_GPIO_OFFSET).expect("failed to get buzzer line");
        let led_line = chip.get_line(LIGHT_GPIO_OFFSET).expect("failed to get led line");

        // set both pinouts to default zero
        // if these fail, it might be b/c there's another process using it
        let buzzer_handle = buzz_line.request(
            LineRequestFlags::OUTPUT, 0, "gpt-alarm").expect("failure requesting buzzer line");

        let led_handle = led_line.request(
            LineRequestFlags::OUTPUT, 0, "gpt-alarm").expect("failure requesting led line");

        Self { buzzer_handle, led_handle }

    }

    pub fn set_buzzer(&self, state: bool) {
        println!("debug: buzzer set to state {}", state);
        self.buzzer_handle.set_value(state as u8).unwrap();
    }
    
    pub fn set_led(&self, state: bool) {
        println!("debug: led set to state {}", state);
        self.led_handle.set_value(state as u8).unwrap();
    }
}

impl Drop for GPIOUtil {
    fn drop(&mut self) {
        self.set_buzzer(false);
        self.set_led(false);
    }
}

#[derive(PartialEq)]
pub enum AlarmState {
    Disarmed,
    Buzzer,
}


pub struct SoundUtil{
    tx: mpsc::Sender<AlarmState>,
}

impl SoundUtil{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<AlarmState>(1);

        tokio::spawn(async {
            Self::async_worker(rx).await;
        });

        SoundUtil {tx}
    }

    async fn async_worker(mut rx: mpsc::Receiver<AlarmState>) {

        println!("info: sound task started");
        let gpio_util = GPIOUtil::build();
        let mut alarm_state = AlarmState::Disarmed;

        loop {

            match rx.try_recv() {
                Ok(update) => {alarm_state = update},
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {break},
            };

            match alarm_state {
                AlarmState::Buzzer => {
                    gpio_util.set_buzzer(true);
                    tokio::time::sleep(Duration::from_millis(125)).await;
                    gpio_util.set_buzzer(false);

                    tokio::time::sleep(Duration::from_millis(125)).await;

                    gpio_util.set_buzzer(true);
                    tokio::time::sleep(Duration::from_millis(125)).await;
                    gpio_util.set_buzzer(false);

                    tokio::time::sleep(Duration::from_millis(750)).await;
                },
                _ => {tokio::time::sleep(Duration::from_millis(500)).await}
            };
        }
        println!("warn: sound async worker exiting")
    }

    pub async fn set_state(&self, state: AlarmState) {
        self.tx.send(state).await.expect("unable to update SoundUtil state");
    }
}
