pub mod detector;
pub mod processor;

pub use detector::{FileTypeDetector, FileType};
pub use processor::{FileProcessor, ProcessedFile, ProcessedContent};

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::Result;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_utils_integration() -> Result<()> {
        // Initialize processor
        let processor = FileProcessor::new()?;

        // Test PDF processing
        let pdf_data = b"%PDF-1.4\nTest PDF";
        let pdf_path = PathBuf::from("test.pdf");
        let processed_pdf = processor.process_file(pdf_data, &pdf_path).await?;
        assert_eq!(processed_pdf.content_type(), "text");

        // Test image processing
        let image_data = [0x89, 0x50, 0x4E, 0x47]; // PNG header
        let image_path = PathBuf::from("test.png");
        let processed_image = processor.process_file(&image_data, &image_path).await?;
        assert_eq!(processed_image.content_type(), "image");

        // Test file type detection
        let detected_type = FileTypeDetector::detect_from_bytes(pdf_data)?;
        assert_eq!(detected_type, FileType::Pdf);

        let mime_type = FileTypeDetector::detect_mime_type(&pdf_path)?;
        assert_eq!(mime_type.type_(), "application");
        assert_eq!(mime_type.subtype(), "pdf");

        Ok(())
    }
}
