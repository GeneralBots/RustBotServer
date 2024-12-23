use gb_core::{Result, Error};
use image::{DynamicImage, ImageFormat, codecs::webp};
use tracing::instrument;

    #[instrument]
pub fn convert_to_format(image_data: &[u8], format: ImageFormat) -> Result<Vec<u8>> {
    let img = image::load_from_memory(image_data)
        .map_err(|e| Error::internal(format!("Failed to load image: {}", e)))?;
    let mut output = Vec::new();
        match format {
        ImageFormat::Jpeg => {
            img.write_to(&mut output, ImageFormat::Jpeg)
                .map_err(|e| Error::internal(format!("JPEG conversion failed: {}", e)))?;
        }
        ImageFormat::Png => {
            img.write_to(&mut output, ImageFormat::Png)
                .map_err(|e| Error::internal(format!("PNG conversion failed: {}", e)))?;
    }
        ImageFormat::WebP => {
            img.write_to(&mut output, ImageFormat::WebP)
                .map_err(|e| Error::internal(format!("WebP conversion failed: {}", e)))?;
}
        _ => return Err(Error::internal("Unsupported format".to_string())),
    }

    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn test_image() -> Vec<u8> {
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buffer = Vec::new();
        img.write_to(&mut buffer, ImageFormat::Png).unwrap();
        buffer
    }

    #[rstest]
    fn test_jpeg_conversion(test_image: Vec<u8>) -> Result<()> {
        let jpeg_data = convert_to_format(&test_image, ImageFormat::Jpeg)?;
        assert!(!jpeg_data.is_empty());
        assert_eq!(image::guess_format(&jpeg_data).unwrap(), ImageFormat::Jpeg);
        Ok(())
    }

    #[rstest]
    fn test_png_conversion(test_image: Vec<u8>) -> Result<()> {
        let png_data = convert_to_format(&test_image, ImageFormat::Png)?;
        assert!(!png_data.is_empty());
        assert_eq!(image::guess_format(&png_data).unwrap(), ImageFormat::Png);
        Ok(())
    }

    #[rstest]
    fn test_webp_conversion(test_image: Vec<u8>) -> Result<()> {
        let webp_data = convert_to_format(&test_image, ImageFormat::WebP)?;
        assert!(!webp_data.is_empty());
        assert_eq!(image::guess_format(&webp_data).unwrap(), ImageFormat::WebP);
        Ok(())
    }
}
