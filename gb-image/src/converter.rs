use std::io::Cursor;
use gb_core::{Result, Error};
use image::{ImageOutputFormat, DynamicImage};
use tracing::instrument;

pub struct ImageConverter;

impl ImageConverter {
    #[instrument]
    pub fn to_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Jpeg(quality))
            .map_err(|e| Error::internal(format!("JPEG conversion failed: {}", e)))?;
        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_png(img: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Png)
            .map_err(|e| Error::internal(format!("PNG conversion failed: {}", e)))?;
        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_webp(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::WebP)
            .map_err(|e| Error::internal(format!("WebP conversion failed: {}", e)))?;
        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_gif(img: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Gif)
            .map_err(|e| Error::internal(format!("GIF conversion failed: {}", e)))?;
        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn test_image() -> DynamicImage {
        DynamicImage::new_rgb8(100, 100)
    }

    #[rstest]
    fn test_jpeg_conversion(test_image: DynamicImage) -> Result<()> {
        let jpeg_data = ImageConverter::to_jpeg(&test_image, 80)?;
        assert!(!jpeg_data.is_empty());
        assert_eq!(image::guess_format(&jpeg_data).unwrap(), image::ImageFormat::Jpeg);
        Ok(())
    }

    #[rstest]
    fn test_png_conversion(test_image: DynamicImage) -> Result<()> {
        let png_data = ImageConverter::to_png(&test_image)?;
        assert!(!png_data.is_empty());
        assert_eq!(image::guess_format(&png_data).unwrap(), image::ImageFormat::Png);
        Ok(())
    }

    #[rstest]
    fn test_webp_conversion(test_image: DynamicImage) -> Result<()> {
        let webp_data = ImageConverter::to_webp(&test_image, 80)?;
        assert!(!webp_data.is_empty());
        Ok(())
    }
}
