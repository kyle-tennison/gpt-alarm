use std::{cell::RefCell, rc::Rc, thread};

use tokio::runtime::Runtime;

use crate::{
    camera::{CamService, FrameFetcher},
    dispatcher::Dispatcher,
};

mod camera;
mod dispatcher;
mod gpio;
mod led;
mod llama;
mod sound;

// main thread (mt)
fn main() {
    // force root
    let is_root = unsafe { libc::getuid() } == 0;
    assert!(is_root, "must be sudo for GPIO");

    // camera servie needs to run on the main thread
    let mut cam_service = CamService::start();
    let frame_fetcher = cam_service.get_fetcher().unwrap();

    // main program runs here
    let at = thread::Builder::new()
        .name("auxillary-thread".to_string())
        .spawn(move || {
            let rt = Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                at_prog(frame_fetcher).await;
            });
        })
        .unwrap();

    // need to do some bs to get this
    let at = Rc::new(RefCell::new(Some(at)));
    let at_closure = at.clone();

    let poll_alive = move || {
        let is_alive = at_closure
            .borrow()
            .as_ref()
            .is_some_and(|f| !f.is_finished());
        println!("info: pulling aux thread for life: {is_alive}");
        is_alive
    };

    cam_service.run_forever(poll_alive);
    eprintln!("error: camera loop exited");

    at.borrow_mut().take().unwrap().join().unwrap()
}

// auxillary thread program. everything besides camera IO runs here
async fn at_prog(frame_fetcher: FrameFetcher) {
    println!("info: start at_prog");
    let dispatcher = Dispatcher::startup(frame_fetcher).await;
    println!("debug: constructed");
    dispatcher.run().await;
}
