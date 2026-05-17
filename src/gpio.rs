use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

const BUZZ_GPIO_OFFSET: u32 = 144;
const LIGHT_GPIO_OFFSET: u32 = 85;
const CHIP_PATH: &str = "/dev/gpiochip0"; // same for both

pub struct GPIOUtil {
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
        println!("debug: gpio shudown");
        self.set_buzzer(false);
        self.set_led(false);
    }
}