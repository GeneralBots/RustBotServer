use gb_core::{Result, Error};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::PathBuf;
use tracing::{error, instrument};

pub struct MediaProcessor {
    pipeline: gst::Pipeline,
}

impl MediaProcessor {
    pub fn new() -> Result<Self> {
        gst::init().map_err(|e| Error::internal(format!("Failed to initialize GStreamer: {}", e)))?;

        let pipeline = gst::Pipeline::new()
            .map_err(|e| Error::internal(format!("Failed to create pipeline: {}", e)))?;

        Ok(Self { pipeline })
    }

    fn setup_pipeline(&mut self) -> Result<()> {
        self.pipeline.set_state(gst::State::Playing)
            .map_err(|e| Error::internal(format!("Failed to start pipeline: {}", e)))?;

        Ok(())
    }

    fn process_messages(&self) -> Result<()> {
        let bus = self.pipeline.bus().unwrap();
        
        while let Some(msg) = bus.timed_pop(gst::ClockTime::from_seconds(1)) {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    error!("Error from {:?}: {} ({:?})", 
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    return Err(Error::internal(format!("Pipeline error: {}", err.error())));
                }
                gst::MessageView::Eos(_) => break,
                _ => ()
            }
        }

        self.pipeline.set_state(gst::State::Null)
            .map_err(|e| Error::internal(format!("Failed to stop pipeline: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self, input_path, output_path))]
    pub async fn transcode(
        &mut self,
        input_path: PathBuf,
        output_path: PathBuf,
        format: &str
    ) -> Result<()> {
        let source = gst::ElementFactory::make("filesrc")
            .map_err(|e| Error::internal(format!("Failed to create source element: {}", e)))?;
        source.set_property("location", input_path.to_str().unwrap());

        let sink = gst::ElementFactory::make("filesink")
            .map_err(|e| Error::internal(format!("Failed to create sink element: {}", e)))?;
        sink.set_property("location", output_path.to_str().unwrap());

        let decoder = match format.to_lowercase().as_str() {
            "mp4" => gst::ElementFactory::make("qtdemux"),
            "webm" => gst::ElementFactory::make("matroskademux"),
            _ => return Err(Error::internal(format!("Unsupported format: {}", format)))
        }.map_err(|e| Error::internal(format!("Failed to create decoder: {}", e)))?;

        self.pipeline.add_many(&[&source, &decoder, &sink])
            .map_err(|e| Error::internal(format!("Failed to add elements: {}", e)))?;

        gst::Element::link_many(&[&source, &decoder, &sink])
            .map_err(|e| Error::internal(format!("Failed to link elements: {}", e)))?;

        self.setup_pipeline()?;
        self.process_messages()
    }
}