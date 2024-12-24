use gb_core::{Result, Error};
use opus::{Encoder, Decoder, Application, Channels};

pub struct AudioProcessor {
    encoder: Encoder,
    decoder: Decoder,
    sample_rate: u32,
    channels: Channels,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32, channels: Channels) -> Result<Self> {
        let encoder = Encoder::new(
            sample_rate,
            channels,
            Application::Audio
        ).map_err(|e| Error::internal(format!("Failed to create Opus encoder: {}", e)))?;

        let decoder = Decoder::new(
            sample_rate,
            channels
        ).map_err(|e| Error::internal(format!("Failed to create Opus decoder: {}", e)))?;

        Ok(Self {
            encoder,
            decoder,
            sample_rate,
            channels,
        })
    }

    pub fn encode(&self, input: &[i16]) -> Result<Vec<u8>> {
        let mut output = vec![0u8; 1024];
        let encoded_size = self.encoder.encode(
            input,
            &mut output
        ).map_err(|e| Error::internal(format!("Failed to encode audio: {}", e)))?;

        output.truncate(encoded_size);
        Ok(output)
    }

    pub fn decode(&self, input: &[u8]) -> Result<Vec<i16>> {
        let max_size = (self.sample_rate as usize / 50) * self.channels.count();
        let mut output = vec![0i16; max_size];

        let decoded_size = self.decoder.decode(
            Some(input),
            &mut output,
            false
        ).map_err(|e| Error::internal(format!("Failed to decode audio: {}", e)))?;

        output.truncate(decoded_size);
        Ok(output)
    }
}