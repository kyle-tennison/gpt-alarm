use std::{sync::mpsc, thread, time::Duration};
use rodio::{OutputStream, Sink, source::SineWave};

pub struct SoundUtil{
    tx: mpsc::Sender<bool>
}

impl SoundUtil {

    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let util = SoundUtil { tx };
        util.spawn_monitor(rx);
        util
    }

    fn spawn_monitor(&self, rx: mpsc::Receiver<bool>) {
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(SineWave::new(950.0));
            sink.set_volume(0.0);

            let mut state = false;
            println!("rust: starting clock loop");
            loop {
                if let Ok(update) = rx.try_recv(){
                    state = update;
                }

                if state {
                    println!("\n\nALARM\nALARM\nALARM\nALARM\nALARM\nALARM\nALARM\nALARM\n\n\n");
                    sink.set_volume(1.0);
                    thread::sleep(Duration::from_millis(100));
                    sink.set_volume(0.0);
                    thread::sleep(Duration::from_millis(100));
                } else {
                    sink.set_volume(0.0);
                    thread::sleep(Duration::from_millis(200));
                }
            }
        });
    }

    pub fn update_state(&self, state: bool){
        self.tx.send(state).unwrap();
    }

}
