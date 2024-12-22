pub mod processor;
pub mod converter;

pub use processor::ImageProcessor;
pub use converter::{ImageConverter, ImageFormat};

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::Result;
    use super::*;
    use gb_core::Result;
    use image::{DynamicImage, Rgba};

    #[tokio::test]
    async fn test_image_processing_integration() -> Result<()> {
        // Initialize components
        let processor = ImageProcessor::new()?;

        // Create test image
        let mut image = DynamicImage::new_rgb8(200, 200);

        // Test image processing operations
        let resized = processor.resize(&image, 100, 100);
        assert_eq!(resized.width(), 100);
        assert_eq!(resized.height(), 100);

        let cropped = processor.crop(&image, 50, 50, 100, 100)?;
        assert_eq!(cropped.width(), 100);
        assert_eq!(cropped.height(), 100);

        let blurred = processor.apply_blur(&image, 1.0);
        let brightened = processor.adjust_brightness(&image, 10);
        let contrasted = processor.adjust_contrast(&image, 1.2);

        // Test text addition
        processor.add_text(
            &mut image,
            "Integration Test",
            10,
            10,
            24.0,
            Rgba([0, 0, 0, 255]),
        )?;

        // Test format conversion
        let webp_data = ImageConverter::to_webp(&image, 80)?;
        let jpeg_data = ImageConverter::to_jpeg(&image, 80)?;
        let png_data = ImageConverter::to_png(&image)?;
        let gif_data = ImageConverter::to_gif(&image)?;


        Ok(())
    }
}
