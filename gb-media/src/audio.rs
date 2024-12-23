use gb_core::{Result, Error};
use opus::{Decoder, Encoder};
use opus::{Decoder, Encoder, Channels, Application};
use std::io::Cursor;
use tracing::{instrument, error};

pub struct AudioProcessor {
    sample_rate: i32,
    channels: i32,
}

impl AudioProcessor {
    pub fn new(sample_rate: i32, channels: i32) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    #[instrument(skip(self, input))]
    pub fn encode(&self, input: &[i16]) -> Result<Vec<u8>> {
        let mut encoder = Encoder::new(
            self.sample_rate,
            if self.channels == 1 {
                opus::Channels::Mono
            } else {
                opus::Channels::Stereo
            },
            opus::Application::Voip,
        ).map_err(|e| Error::internal(format!("Failed to create Opus encoder: {}", e)))?;
            u32::try_from(self.sample_rate).map_err(|e| Error::internal(format!("Invalid sample rate: {}", e)))?,
            Channels::Mono,
            Application::Voip
        ).map_err(|e| Error::internal(format!("Failed to create Opus encoder: {}", e)))?;

        let mut output = vec![0u8; 1024];
        let encoded_len = encoder.encode(input, &mut output)
            .map_err(|e| Error::internal(format!("Failed to encode audio: {}", e)))?;

        output.truncate(encoded_len);
        Ok(output)
        encoder.encode(input)
            .map_err(|e| Error::internal(format!("Failed to encode audio: {}", e)))
    }

    #[instrument(skip(self, input))]
    pub fn decode(&self, input: &[u8]) -> Result<Vec<i16>> {
        let mut decoder = Decoder::new(
            self.sample_rate,
            if self.channels == 1 {
                opus::Channels::Mono
            } else {
                opus::Channels::Stereo
            },
        ).map_err(|e| Error::internal(format!("Failed to create Opus decoder: {}", e)))?;
            u32::try_from(self.sample_rate).map_err(|e| Error::internal(format!("Invalid sample rate: {}", e)))?,
            Channels::Mono
        ).map_err(|e| Error::internal(format!("Failed to create Opus decoder: {}", e)))?;

        let mut output = vec![0i16; 1024];
        let decoded_len = decoder.decode(input, &mut output, false)
            .map_err(|e| Error::internal(format!("Failed to decode audio: {}", e)))?;

        output.truncate(decoded_len);
        Ok(output)
        decoder.decode(input)
            .map_err(|e| Error::internal(format!("Failed to decode audio: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn audio_processor() -> AudioProcessor {
        AudioProcessor::new(48000, 2)
    }

    #[fixture]
    fn test_audio() -> Vec<i16> {
        // Generate 1 second of 440Hz sine wave
        let sample_rate = 48000;
        let frequency = 440.0;
        let duration = 1.0;
        
        (0..sample_rate)
            .flat_map(|i| {
                let t = i as f32 / sample_rate as f32;
                let value = (2.0 * std::f32::consts::PI * frequency * t).sin();
                let sample = (value * i16::MAX as f32) as i16;
                vec![sample, sample] // Stereo
                vec![sample, sample]
            })
            .collect()
    }

    #[rstest]
    fn test_encode_decode(audio_processor: AudioProcessor, test_audio: Vec<i16>) {
        let encoded = audio_processor.encode(&test_audio).unwrap();
        let decoded = audio_processor.decode(&encoded).unwrap();

        // Verify basic properties
        assert!(!encoded.is_empty());
        assert!(!decoded.is_empty());

        // Opus is lossy, so we can't compare exact values
        // But we can verify the length is the same
        assert_eq!(decoded.len(), test_audio.len());
    }
}
