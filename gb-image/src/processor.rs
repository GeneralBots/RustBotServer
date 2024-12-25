use gb_core::{Error, Result};
use image::{DynamicImage, ImageOutputFormat, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::io::Cursor;
use tesseract::Tesseract;
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;


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


    pub fn resize(&self, image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        image.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    pub fn crop(&self, image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Result<DynamicImage> {
        if x + width > image.width() || y + height > image.height() {
            return Err(Error::internal("Crop dimensions exceed image bounds".to_string()));
        }
        Ok(image.crop_imm(x, y, width, height))
    }

    pub fn apply_blur(&self, image: &DynamicImage, sigma: f32) -> DynamicImage {
        image.blur(sigma)
    }

    pub fn adjust_brightness(&self, image: &DynamicImage, value: i32) -> DynamicImage {
        image.brighten(value)
    }

    pub fn adjust_contrast(&self, image: &DynamicImage, value: f32) -> DynamicImage {
        image.adjust_contrast(value)
    }

    pub fn add_text(
        &self,
        image: &mut DynamicImage,
        text: &str,
        x: i32,
        y: i32,
        size: f32,
        color: Rgba<u8>,
    ) -> Result<()> {
        // Load the font file from assets (downloaded in build.rs)
        let font_data = fs::read("assets/DejaVuSans.ttf")
            .map_err(|e| Error::internal(format!("Failed to load font: {}", e)))?;

        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| Error::internal("Failed to parse font data".to_string()))?;

        let scale = Scale::uniform(size);
        let image_buffer = image.as_mut_rgba8()
            .ok_or_else(|| Error::internal("Failed to convert image to RGBA".to_string()))?;

        draw_text_mut(
            image_buffer,
            color,
            x,
            y,
            scale,
            &font,
            text
        );

        Ok(())
    }


}