use async_trait::async_trait;
use gb_core::{Result, Error};
use gstreamer as gst;
use std::path::PathBuf;
use tracing::{instrument, error};

pub struct MediaProcessor {
    pipeline: gst::Pipeline,
}

impl MediaProcessor {
    pub fn new() -> Result<Self> {
        gst::init().map_err(|e| Error::Internal(format!("Failed to initialize GStreamer: {}", e)))?;
        
        let pipeline = gst::Pipeline::new(None);
        Ok(Self { pipeline })
    }

    #[instrument(skip(self, input_path, output_path))]
    pub async fn transcode(
        &self,
        input_path: PathBuf,
        output_path: PathBuf,
        format: &str,
    ) -> Result<()> {
        let src = gst::ElementFactory::make("filesrc")
            .property("location", input_path.to_str().unwrap())
            .build()
            .map_err(|e| Error::Internal(format!("Failed to create source element: {}", e)))?;

        let sink = gst::ElementFactory::make("filesink")
            .property("location", output_path.to_str().unwrap())
            .build()
            .map_err(|e| Error::Internal(format!("Failed to create sink element: {}", e)))?;

        let decoder = match format {
            "h264" => gst::ElementFactory::make("h264parse").build(),
            "opus" => gst::ElementFactory::make("opusparse").build(),
            _ => return Err(Error::InvalidInput(format!("Unsupported format: {}", format))),
        }.map_err(|e| Error::Internal(format!("Failed to create decoder: {}", e)))?;

        self.pipeline.add_many(&[&src, &decoder, &sink])
            .map_err(|e| Error::Internal(format!("Failed to add elements: {}", e)))?;

        gst::Element::link_many(&[&src, &decoder, &sink])
            .map_err(|e| Error::Internal(format!("Failed to link elements: {}", e)))?;

        self.pipeline.set_state(gst::State::Playing)
            .map_err(|e| Error::Internal(format!("Failed to start pipeline: {}", e)))?;

        let bus = self.pipeline.bus().unwrap();
        
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Error(err) => {
                    error!("Error from {:?}: {} ({:?})", 
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    return Err(Error::Internal(format!("Pipeline error: {}", err.error())));
                }
                MessageView::Eos(_) => break,
                _ => (),
            }
        }

        self.pipeline.set_state(gst::State::Null)
            .map_err(|e| Error::Internal(format!("Failed to stop pipeline: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self, input_path))]
    pub async fn extract_metadata(&self, input_path: PathBuf) -> Result<MediaMetadata> {
        let src = gst::ElementFactory::make("filesrc")
            .property("location", input_path.to_str().unwrap())
            .build()
            .map_err(|e| Error::Internal(format!("Failed to create source element: {}", e)))?;

        let decodebin = gst::ElementFactory::make("decodebin").build()
            .map_err(|e| Error::Internal(format!("Failed to create decodebin: {}", e)))?;

        self.pipeline.add_many(&[&src, &decodebin])
            .map_err(|e| Error::Internal(format!("Failed to add elements: {}", e)))?;

        gst::Element::link_many(&[&src, &decodebin])
            .map_err(|e| Error::Internal(format!("Failed to link elements: {}", e)))?;

        let mut metadata = MediaMetadata::default();

        decodebin.connect_pad_added(move |_, pad| {
            let caps = pad.current_caps().unwrap();
            let structure = caps.structure(0).unwrap();
            
            match structure.name() {
                "video/x-raw" => {
                    if let Ok(width) = structure.get::<i32>("width") {
                        metadata.width = Some(width);
                    }
                    if let Ok(height) = structure.get::<i32>("height") {
                        metadata.height = Some(height);
                    }
                    if let Ok(framerate) = structure.get::<gst::Fraction>("framerate") {
                        metadata.framerate = Some(framerate.numer() as f64 / framerate.denom() as f64);
                    }
                },
                "audio/x-raw" => {
                    if let Ok(channels) = structure.get::<i32>("channels") {
                        metadata.channels = Some(channels);
                    }
                    if let Ok(rate) = structure.get::<i32>("rate") {
                        metadata.sample_rate = Some(rate);
                    }
                },
                _ => (),
            }
        });

        self.pipeline.set_state(gst::State::Playing)
            .map_err(|e| Error::Internal(format!("Failed to start pipeline: {}", e)))?;

        let bus = self.pipeline.bus().unwrap();
        
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Error(err) => {
                    error!("Error from {:?}: {} ({:?})", 
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    return Err(Error::Internal(format!("Pipeline error: {}", err.error())));
                }
                MessageView::Eos(_) => break,
                _ => (),
            }
        }

        self.pipeline.set_state(gst::State::Null)
            .map_err(|e| Error::Internal(format!("Failed to stop pipeline: {}", e)))?;

        Ok(metadata)
    }
}

#[derive(Debug, Default)]
pub struct MediaMetadata {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub channels: Option<i32>,
    pub sample_rate: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use rstest::*;

    #[fixture]
    fn media_processor() -> MediaProcessor {
        MediaProcessor::new().unwrap()
    }

    #[fixture]
    fn test_video_path() -> PathBuf {
        PathBuf::from("test_data/test.mp4")
    }

    #[rstest]
    #[tokio::test]
    async fn test_transcode(media_processor: MediaProcessor, test_video_path: PathBuf) {
        let output_path = PathBuf::from("test_data/output.mp4");
        
        let result = media_processor.transcode(
            test_video_path,
            output_path.clone(),
            "h264",
        ).await;

        assert!(result.is_ok());
        assert!(output_path.exists());
        std::fs::remove_file(output_path).unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_extract_metadata(media_processor: MediaProcessor, test_video_path: PathBuf) {
        let metadata = media_processor.extract_metadata(test_video_path).await.unwrap();
        
        assert!(metadata.width.is_some());
        assert!(metadata.height.is_some());
        assert!(metadata.framerate.is_some());
    }
}
