use gb_core::{Result, Error};
use image::{
    DynamicImage, ImageOutputFormat,
    codecs::{webp, jpeg, png, gif},
};
use std::io::Cursor;
use tracing::instrument;

pub struct ImageConverter;

impl ImageConverter {
    #[instrument]
    pub fn to_webp(image: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        let encoder = webp::WebPEncoder::new_with_quality(&mut buffer, quality as f32);
        
        encoder.encode(
            image.as_bytes(),
            image.width(),
            image.height(),
            image.color(),
        ).map_err(|e| Error::Internal(format!("WebP conversion failed: {}", e)))?;

        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_jpeg(image: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Jpeg(quality))
            .map_err(|e| Error::Internal(format!("JPEG conversion failed: {}", e)))?;

        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_png(image: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Png)
            .map_err(|e| Error::Internal(format!("PNG conversion failed: {}", e)))?;

        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn to_gif(image: &DynamicImage) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Gif)
            .map_err(|e| Error::Internal(format!("GIF conversion failed: {}", e)))?;

        Ok(buffer.into_inner())
    }

    #[instrument]
    pub fn get_format(data: &[u8]) -> Result<ImageFormat> {
        let format = image::guess_format(data)
            .map_err(|e| Error::Internal(format!("Failed to determine format: {}", e)))?;

        match format {
            image::ImageFormat::WebP => Ok(ImageFormat::WebP),
            image::ImageFormat::Jpeg => Ok(ImageFormat::Jpeg),
            image::ImageFormat::Png => Ok(ImageFormat::Png),
            image::ImageFormat::Gif => Ok(ImageFormat::Gif),
            _ => Err(Error::Internal("Unsupported format".to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    WebP,
    Jpeg,
    Png,
    Gif,
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
    fn test_webp_conversion(test_image: DynamicImage) -> Result<()> {
        let webp_data = ImageConverter::to_webp(&test_image, 80)?;
        assert!(!webp_data.is_empty());
        assert_eq!(ImageConverter::get_format(&webp_data)?, ImageFormat::WebP);
        Ok(())
    }

    #[rstest]
    fn test_jpeg_conversion(test_image: DynamicImage) -> Result<()> {
        let jpeg_data = ImageConverter::to_jpeg(&test_image, 80)?;
        assert!(!jpeg_data.is_empty());
        assert_eq!(ImageConverter::get_format(&jpeg_data)?, ImageFormat::Jpeg);
        Ok(())
    }

    #[rstest]
    fn test_png_conversion(test_image: DynamicImage) -> Result<()> {
        let png_data = ImageConverter::to_png(&test_image)?;
        assert!(!png_data.is_empty());
        assert_eq!(ImageConverter::get_format(&png_data)?, ImageFormat::Png);
        Ok(())
    }

    #[rstest]
    fn test_gif_conversion(test_image: DynamicImage) -> Result<()> {
        let gif_data = ImageConverter::to_gif(&test_image)?;
        assert!(!gif_data.is_empty());
        assert_eq!(ImageConverter::get_format(&gif_data)?, ImageFormat::Gif);
        Ok(())
    }
}
