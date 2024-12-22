use gb_core::{Result, Error};
use image::{
    DynamicImage, ImageBuffer, Rgba, GenericImageView,
    imageops::{blur, brighten, contrast},
};
use imageproc::{
    drawing::{draw_text_mut, draw_filled_rect_mut},
    rect::Rect,
};
use rusttype::{Font, Scale};
use std::path::Path;
use tracing::{instrument, error};

pub struct ImageProcessor {
    default_font: Font<'static>,
}

impl ImageProcessor {
    pub fn new() -> Result<Self> {
        let font_data = include_bytes!("../assets/DejaVuSans.ttf");
        let font = Font::try_from_bytes(font_data)
            .ok_or_else(|| Error::Internal("Failed to load font".to_string()))?;

        Ok(Self {
            default_font: font,
        })
    }

    #[instrument(skip(self, image_data))]
    pub fn load_image(&self, image_data: &[u8]) -> Result<DynamicImage> {
        image::load_from_memory(image_data)
            .map_err(|e| Error::Internal(format!("Failed to load image: {}", e)))
    }

    #[instrument(skip(self, image))]
    pub fn save_image(&self, image: &DynamicImage, path: &Path) -> Result<()> {
        image.save(path)
            .map_err(|e| Error::Internal(format!("Failed to save image: {}", e)))
    }

    #[instrument(skip(self, image))]
    pub fn resize(&self, image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        image.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    #[instrument(skip(self, image))]
    pub fn crop(&self, image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Result<DynamicImage> {
        image.crop_imm(x, y, width, height)
            .map_err(|e| Error::Internal(format!("Failed to crop image: {}", e)))
            .map(|img| img.to_owned())
    }

    #[instrument(skip(self, image))]
    pub fn apply_blur(&self, image: &DynamicImage, sigma: f32) -> DynamicImage {
        let mut img = image.clone();
        blur(&mut img, sigma);
        img
    }

    #[instrument(skip(self, image))]
    pub fn adjust_brightness(&self, image: &DynamicImage, value: i32) -> DynamicImage {
        let mut img = image.clone();
        brighten(&mut img, value);
        img
    }

    #[instrument(skip(self, image))]
    pub fn adjust_contrast(&self, image: &DynamicImage, value: f32) -> DynamicImage {
        let mut img = image.clone();
        contrast(&mut img, value);
        img
    }

    #[instrument(skip(self, image, text))]
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
        image::imageops::overlay(image, watermark, x, y);
        Ok(())
    }

    #[instrument(skip(self, image))]
    pub fn extract_text(&self, image: &DynamicImage) -> Result<String> {
        use tesseract::Tesseract;

        let mut temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| Error::Internal(format!("Failed to create temp file: {}", e)))?;
            
        image.save(&temp_file)
            .map_err(|e| Error::Internal(format!("Failed to save temp image: {}", e)))?;

        let text = Tesseract::new(None, Some("eng"))
            .map_err(|e| Error::Internal(format!("Failed to initialize Tesseract: {}", e)))?
            .set_image_from_path(temp_file.path())
            .map_err(|e| Error::Internal(format!("Failed to set image: {}", e)))?
            .recognize()
            .map_err(|e| Error::Internal(format!("Failed to recognize text: {}", e)))?
            .get_text()
            .map_err(|e| Error::Internal(format!("Failed to get text: {}", e)))?;

        Ok(text)
    }

    #[instrument(skip(self, image))]
    pub fn detect_faces(&self, image: &DynamicImage) -> Result<Vec<Rect>> {
        use opencv::{
            core,
            objdetect::CascadeClassifier,
            prelude::*,
            types::VectorOfRect,
        };

        let mut classifier = CascadeClassifier::new(&format!(
            "{}/haarcascade_frontalface_default.xml",
            std::env::var("OPENCV_DATA_PATH")
                .unwrap_or_else(|_| "/usr/share/opencv4".to_string())
        )).map_err(|e| Error::Internal(format!("Failed to load classifier: {}", e)))?;

        let mut img = core::Mat::new_rows_cols_with_default(
            image.height() as i32,
            image.width() as i32,
            core::CV_8UC3,
            core::Scalar::all(0.0),
        ).map_err(|e| Error::Internal(format!("Failed to create Mat: {}", e)))?;

        // Convert DynamicImage to OpenCV Mat
        let rgb = image.to_rgb8();
        unsafe {
            img.set_data(rgb.as_raw().as_ptr() as *mut u8, core::CV_8UC3)?;
        }

        let mut faces = VectorOfRect::new();
        classifier.detect_multi_scale(
            &img,
            &mut faces,
            1.1,
            3,
            0,
            core::Size::new(30, 30),
            core::Size::new(0, 0),
        ).map_err(|e| Error::Internal(format!("Face detection failed: {}", e)))?;

        Ok(faces.iter().map(|r| Rect::at(r.x, r.y).of_size(r.width as u32, r.height as u32))
            .collect())
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

    #[rstest]
    fn test_detect_faces(processor: ImageProcessor, test_image: DynamicImage) -> Result<()> {
        let faces = processor.detect_faces(&test_image)?;
        assert!(faces.is_empty()); // Test image has no faces
        Ok(())
    }
}
