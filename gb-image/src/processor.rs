use gb_core::{Error, Result};
use image::{DynamicImage, ImageOutputFormat};
use std::io::Cursor;
use tesseract::Tesseract;
use tempfile::NamedTempFile;
use std::io::Write;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract_text(&self, image: &DynamicImage) -> Result<String> {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| Error::internal(format!("Failed to create temp file: {}", e)))?;
        
        // Convert image to PNG and write to temp file
        let mut cursor = Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageOutputFormat::Png)
            .map_err(|e| Error::internal(format!("Failed to encode image: {}", e)))?;
        
        temp_file.write_all(&cursor.into_inner())
            .map_err(|e| Error::internal(format!("Failed to write to temp file: {}", e)))?;

        // Initialize Tesseract and process image
        let api = Tesseract::new(None, Some("eng"))
            .map_err(|e| Error::internal(format!("Failed to initialize Tesseract: {}", e)))?;

        api.set_image(temp_file.path().to_str().unwrap())
            .map_err(|e| Error::internal(format!("Failed to set image: {}", e)))?
            .recognize()
            .map_err(|e| Error::internal(format!("Failed to recognize text: {}", e)))?
            .get_text()
            .map_err(|e| Error::internal(format!("Failed to get text: {}", e)))
    }
}