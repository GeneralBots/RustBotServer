use gb_core::{Result, Error};
use image::{
    DynamicImage, Rgba,
};
use imageproc::{
    drawing::draw_text_mut,
};
use rusttype::{Font, Scale};
use std::path::Path;
use tracing::instrument;
use std::convert::TryInto;

pub struct ProcessingOptions {
    pub crop: Option<CropParams>,
    pub watermark: Option<DynamicImage>,
    pub x: i32,
    pub y: i32,
}

pub struct CropParams {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct ImageProcessor {
    default_font: Font<'static>,
}

impl ImageProcessor {
    pub fn new() -> Result<Self> {
        let font_data = include_bytes!("../assets/DejaVuSans.ttf");
        let font = Font::try_from_bytes(font_data)
            .ok_or_else(|| Error::internal("Failed to load font"))?;

        Ok(Self {
            default_font: font,
        })
    }

    pub fn process_image(&self, mut image: DynamicImage, options: &ProcessingOptions) -> Result<DynamicImage> {
        if let Some(crop) = &options.crop {
            let cropped = image.crop_imm(
                crop.x,
                crop.y,
                crop.width,
                crop.height
            );
            image = cropped;
        }

        if let Some(watermark) = &options.watermark {
            let x: i64 = options.x.try_into().map_err(|_| Error::internal("Invalid x coordinate"))?;
            let y: i64 = options.y.try_into().map_err(|_| Error::internal("Invalid y coordinate"))?;
            image::imageops::overlay(&mut image, watermark, x, y);
        }

        Ok(image)
    }

    #[instrument(skip(self, image_data))]
    pub fn load_image(&self, image_data: &[u8]) -> Result<DynamicImage> {
        image::load_from_memory(image_data)
            .map_err(|e| Error::internal(format!("Failed to load image: {}", e)))
    }

    #[instrument(skip(self, image))]
    pub fn save_image(&self, image: &DynamicImage, path: &Path) -> Result<()> {
        image.save(path)
            .map_err(|e| Error::internal(format!("Failed to save image: {}", e)))
    }

    #[instrument(skip(self, image))]
    pub fn crop(&self, image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Result<DynamicImage> {
        Ok(image.crop_imm(x, y, width, height))
    }

    #[instrument(skip(self, image))]
    pub fn add_text(
        &self,
        image: &mut DynamicImage,
        text: &str,
        x: i32,
        y: i32,
        scale: f32,
        color: Rgba<u8>,
    ) -> Result<()> {
        let scale = Scale::uniform(scale);
        
        let mut img = image.to_rgba8();
        draw_text_mut(
            &mut img,
            color,
            x,
            y,
            scale,
            &self.default_font,
            text,
        );

        *image = DynamicImage::ImageRgba8(img);
        Ok(())
    }

    #[instrument(skip(self, image))]
    pub fn add_watermark(
        &self,
        image: &mut DynamicImage,
        watermark: &DynamicImage,
        x: u32,
        y: u32,
    ) -> Result<()> {
        let x: i64 = x.try_into().map_err(|_| Error::internal("Invalid x coordinate"))?;
        let y: i64 = y.try_into().map_err(|_| Error::internal("Invalid y coordinate"))?;
        image::imageops::overlay(image, watermark, x, y);
        Ok(())
    }

    #[instrument(skip(self, image))]
    pub fn extract_text(&self, image: &DynamicImage) -> Result<String> {
        use tesseract::Tesseract;

        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| Error::internal(format!("Failed to create temp file: {}", e)))?;
            
        image.save(&temp_file)
            .map_err(|e| Error::internal(format!("Failed to save temp image: {}", e)))?;

        let mut api = Tesseract::new(None, Some("eng"))
            .map_err(|e| Error::internal(format!("Failed to initialize Tesseract: {}", e)))?;

        api.set_image(temp_file.path().to_str().unwrap())
            .map_err(|e| Error::internal(format!("Failed to set image: {}", e)))?;

        api.recognize()
            .map_err(|e| Error::internal(format!("Failed to recognize text: {}", e)))?;

        api.get_text()
            .map_err(|e| Error::internal(format!("Failed to get text: {}", e)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::path::PathBuf;

    #[fixture]
    fn processor() -> ImageProcessor {
        ImageProcessor::new().unwrap()
    }

    #[fixture]
    fn test_image() -> DynamicImage {
        DynamicImage::new_rgb8(100, 100)
    }

    #[rstest]
    fn test_resize(processor: ImageProcessor, test_image: DynamicImage) {
        let resized = processor.resize(&test_image, 50, 50);
        assert_eq!(resized.width(), 50);
        assert_eq!(resized.height(), 50);
    }

    #[rstest]
    fn test_crop(processor: ImageProcessor, test_image: DynamicImage) -> Result<()> {
        let cropped = processor.crop(&test_image, 25, 25, 50, 50)?;
        assert_eq!(cropped.width(), 50);
        assert_eq!(cropped.height(), 50);
        Ok(())
    }

    #[rstest]
    fn test_add_text(processor: ImageProcessor, mut test_image: DynamicImage) -> Result<()> {
        processor.add_text(
            &mut test_image,
            "Test",
            10,
            10,
            12.0,
            Rgba([255, 255, 255, 255]),
        )?;
        Ok(())
    }

    #[rstest]
    fn test_extract_text(processor: ImageProcessor, mut test_image: DynamicImage) -> Result<()> {
        processor.add_text(
            &mut test_image,
            "Test OCR",
            10,
            10,
            24.0,
            Rgba([0, 0, 0, 255]),
        )?;

        let text = processor.extract_text(&test_image)?;
        assert!(text.contains("Test OCR"));
        Ok(())
    }
}