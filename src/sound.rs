use std::{sync::{Mutex, mpsc}, thread, time::Duration};

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
            let mut state = false;
            println!("rust: starting clock loop");
            loop {
                if let Ok(update) = rx.try_recv(){
                    state = update;
                }

                if state {
                    Self::playsound();
                    thread::sleep(Duration::from_millis(250));
                }
            } 
        });
    }

    fn playsound(){
        let _ = subprocess::Exec::cmd("afplay")
            .arg("/System/Library/Sounds/Ping.aiff")
            .start();
    }

    pub fn update_state(&self, state: bool){
        self.tx.send(state).unwrap();
    }



}