extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
use gst::{
    MessageType,
    prelude::{ElementExt, GstBinExtManual, GstObjectExt},
};
use gstreamer::{glib::object::Cast, prelude::ElementExtManual};
use std::{sync::{Arc, Mutex, mpsc}, time::{Duration, Instant}};
use tokio::{
    io::AsyncWriteExt,
};

pub struct CamService {
    pipeline: gst::Pipeline,
    ff: Option<FrameFetcher>
}

impl CamService {
    /// Setup and start gst stream
    pub fn start() -> Self {
        gst::init().unwrap();

        let source = gst::ElementFactory::make("nvarguscamerasrc")
            .name("source")
            .build()
            .expect("could not create source element.");

        let enc = gst::ElementFactory::make("nvjpegenc")
            .name("enc")
            .build()
            .expect("could not create encoder.");

        let caps = gst::Caps::builder("image/jpeg")
            .field("width", 854)
            .field("height", 480)
            .build();



        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let req_frame_flag = Arc::new(Mutex::new(false));
        let req_frame_flag_closure = req_frame_flag.clone();

        let callbacks = gst_app::AppSinkCallbacks::builder()
            .new_sample(move |aps| {

                match req_frame_flag_closure.try_lock() {
                    Ok(mut guard) => {
                        if *guard {
                            *guard = false;
                            let sample = aps.pull_sample().expect("failed to pull sample");
                            tx.send(Self::extract_bytes(sample)).unwrap();
                            println!("debug: sent sample");
                        }
                    }
                    Err(_) => {
                        println!("debug: mutex crash, skipping image");
                    },
                }

                Ok(gst::FlowSuccess::Ok)
            })
            .build();

        let appsink = gst_app::AppSink::builder()
            .callbacks(callbacks)
            .name("appsink")
            .max_buffers(2)
            .drop(true)
            .build()
            .dynamic_cast().unwrap();


        let pipeline = gst::Pipeline::with_name("pipeline");

        pipeline
            .add_many([&source, &enc, &appsink])
            .expect("unable to add elements to pipeline");

        source
            .link(&enc)
            .expect("failed to link src to jpg encoder");
        enc.link_filtered(&appsink, &caps)
            .expect("failed to link encoder to sink");

        println!("info: gst pipeline constructed successfully");

        let ff = FrameFetcher { req_frame_flag: req_frame_flag, frame_rx: rx };

        CamService { pipeline, ff: Some(ff) }
    }

    pub fn extract_bytes(sample: gst::Sample) -> Vec<u8> {
        sample
        .buffer()
        .and_then(|buffer| buffer.map_readable().ok())
        .map(|map| map.as_slice().to_vec())
        .unwrap_or_default()
    }

    pub fn get_fetcher(&mut self) -> Option<FrameFetcher> {
        self.ff.take()
    }

    pub fn run_forever(&self) {
        self.pipeline
            .set_state(gst::State::Playing)
            .expect("failed to start playing pipeline");

        println!("info: pipeline started");

        let bus = self.pipeline.bus().unwrap();
        for msg in bus.iter_timed_filtered(
            gst::ClockTime::NONE,
            &[MessageType::Error, MessageType::Eos],
        ) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Error(err) => {
                    eprintln!(
                        "Error received from element {:?}: {}",
                        err.src().map(|s| s.path_string()),
                        err.error()
                    );
                    eprintln!("Debugging information: {:?}", err.debug());
                    break;
                }
                MessageView::Eos(..) => break,
                _ => (),
            }
        }

        self.pipeline
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");
    }
}

impl Drop for CamService {
    fn drop(&mut self) {
        println!("info: CamService starting cleaning up")
    }
}




pub struct FrameFetcher {
    req_frame_flag: Arc<Mutex<bool>>,
    frame_rx: mpsc::Receiver<Vec<u8>>,
}

impl FrameFetcher {

    pub fn fetch_frame_bytes(&self) -> Vec<u8>{
        let fetch_start = Instant::now();
        // requrest frame
        {
            let mut guard = self.req_frame_flag.lock().unwrap();
            *guard = true;
        }
    
        // wait for inbound frame
        let jpeg_buffer = self.frame_rx.recv_timeout(Duration::from_secs(10)).expect("rame fetch timed out");    
            
        let elapsed = Instant::now() - fetch_start;
        println!("info: collected jpeg in {:.8} seconds", elapsed.as_secs_f32());

        jpeg_buffer
    }

    // Writes image bytes to a jpg file
    pub async fn save_photo_bytes(&self, filename: &str, bytes: &[u8]) {
        let mut image_file = tokio::fs::File::create(filename).await.unwrap();
        image_file
            .write_all(&bytes)
            .await
            .expect("failed to save image");

        println!("info: saved photo. {:?}", image_file);
    }

    /// Takes and saves a photo to a file
    pub async fn save_photo(&self, filename: &str) {
        let bytes = self.fetch_frame_bytes();
        self.save_photo_bytes(filename, &bytes).await;
    }
}