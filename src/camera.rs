extern crate gstreamer as gst;
use std::{io::{Read, Write}, thread, time::{Duration, Instant}};
use gst::{MessageType, glib::{object::ObjectExt, value::ToValue}, prelude::{ElementExt, GstBinExtManual, GstObjectExt}};
use gstreamer::prelude::ElementExtManual;

const BUFFER_LOCATION: &str = "/tmp/gpt-alarm";
const JPEG_EOF: [u8; 2] = *b"\xFF\xD8";

pub struct Camera {
    pipeline: gst::Pipeline
}

impl Camera {

    pub fn build() -> Self {

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
            .field("width", 1280)
            .field("height", 720)
            .build();

        let sink = gst::ElementFactory::make("filesink")
            .name("sink")
            .property_from_str("location", BUFFER_LOCATION)
            .property_from_str("buffer-mode", "2")
            .build()
            .expect("could not create sink element.");

        let pipeline = gst::Pipeline::with_name("pipeline");
        
        pipeline.add_many([
            &source,
            &enc,
            &sink
        ]).expect("unable to add elements to pipeline");
        
        source.link(&enc).expect("failed to link src to jpg encoder");
        enc.link_filtered(&sink, &caps).expect("failed to link encoder to sink");

        println!("info: gst pipeline constructed successfully");

        Camera {pipeline}
    }
    
    pub fn run_forever(&self) {
        self.pipeline.set_state(gst::State::Playing)
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


    pub fn fetch_frame_bytes() -> Vec<u8>{
        let mut jpeg_buffer: Vec<u8> = Vec::with_capacity(250_000);

        let mut fifo = std::fs::File::open(BUFFER_LOCATION).expect("could not find fifo file");
        
        let finder = memchr::memmem::Finder::new(&JPEG_EOF);
        let mut stack_buf: [u8; 4096] = [0;4096];
        let mut in_frame = false; // flag to see if we're in the frame we want

        let start = Instant::now();
        println!("info: waiting for jpeg");
        'fifo: loop {
            fifo.read_exact(&mut stack_buf).expect("Unable to read from fifo");
            
            // this triggers if the diliminer is found
            if let Some(pos) = finder.find(&stack_buf) {
                if !in_frame{
                    jpeg_buffer.extend_from_slice(&stack_buf[pos..]); // throw out anything before frame
                    in_frame = true;
                }
                else {
                    // if we find a second delimiter, that means we are done reading 
                    jpeg_buffer.extend_from_slice(&stack_buf[..pos]);
                    break 'fifo;
                }
            }
            else{
                if in_frame {
                    jpeg_buffer.extend_from_slice(&stack_buf);
                }
            }
        }
        let elapsed = Instant::now() - start;
        println!("info: collected jepg in {:.8} seconds", elapsed.as_secs_f32());

        jpeg_buffer
    }

}