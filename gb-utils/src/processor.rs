use gb_core::{Result, Error};
use gb_document::{PdfProcessor, WordProcessor, ExcelProcessor};
use gb_image::{ImageProcessor, ImageConverter};
use super::detector::{FileTypeDetector, FileType};
use std::path::Path;
use tracing::{instrument, error};
use uuid::Uuid;

pub struct FileProcessor {
    image_processor: ImageProcessor,
}

impl FileProcessor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            image_processor: ImageProcessor::new()?,
        })
    }

    #[instrument(skip(self, data))]
    pub async fn process_file(&self, data: &[u8], path: &Path) -> Result<ProcessedFile> {
        let file_type = FileTypeDetector::detect_from_bytes(data)?;
        let mime_type = FileTypeDetector::detect_mime_type(path)?;

        match file_type {
            FileType::Pdf => {
                let text = PdfProcessor::extract_text(data)?;
                Ok(ProcessedFile {
                    id: Uuid::new_v4(),
                    original_name: path.file_name().unwrap().to_string_lossy().to_string(),
                    mime_type,
                    content: ProcessedContent::Text(text),
                })
            }
            FileType::Word => {
                let text = WordProcessor::extract_text(data)?;
                Ok(ProcessedFile {
                    id: Uuid::new_v4(),
                    original_name: path.file_name().unwrap().to_string_lossy().to_string(),
                    mime_type,
                    content: ProcessedContent::Text(text),
                })
            }
            FileType::Excel => {
                let json = ExcelProcessor::extract_as_json(data)?;
                Ok(ProcessedFile {
                    id: Uuid::new_v4(),
                    original_name: path.file_name().unwrap().to_string_lossy().to_string(),
                    mime_type,
                    content: ProcessedContent::Json(json),
                })
            }
            FileType::Png | FileType::Jpeg | FileType::WebP => {
                let image = self.image_processor.load_image(data)?;
                let text = self.image_processor.extract_text(&image)?;
                Ok(ProcessedFile {
                    id: Uuid::new_v4(),
                    original_name: path.file_name().unwrap().to_string_lossy().to_string(),
                    mime_type,
                    content: ProcessedContent::Image {
                        text,
                        width: image.width(),
                        height: image.height(),
                    },
                })
            }
        }
    }

    #[instrument(skip(self, file))]
    pub async fn convert_file(
        &self,
        file: &ProcessedFile,
        target_type: FileType,
    ) -> Result<Vec<u8>> {
        match (&file.content, target_type) {
            (ProcessedContent::Image { .. }, FileType::Png) => {
                let image = self.image_processor.load_image(file.raw_data())?;
                ImageConverter::to_png(&image)
            }
            (ProcessedContent::Image { .. }, FileType::Jpeg) => {
                let image = self.image_processor.load_image(file.raw_data())?;
                ImageConverter::to_jpeg(&image, 80)
            }
   EOL
   
# Continuing gb-utils/src/processor.rs
cat >> gb-utils/src/processor.rs << 'EOL'
            (ProcessedContent::Image { .. }, FileType::WebP) => {
                let image = self.image_processor.load_image(file.raw_data())?;
                ImageConverter::to_webp(&image, 80)
            }
            (ProcessedContent::Text(text), FileType::Pdf) => {
                let doc = PdfProcessor::create_document(text)?;
                Ok(doc)
            }
            (ProcessedContent::Text(text), FileType::Word) => {
                let doc = WordProcessor::create_document(text)?;
                Ok(doc)
            }
            (ProcessedContent::Json(json), FileType::Excel) => {
                let data: Vec<Vec<String>> = serde_json::from_value(json.clone())?;
                let headers: Vec<&str> = data[0].iter().map(|s| s.as_str()).collect();
                ExcelProcessor::create_excel(&headers, &data[1..])
            }
            _ => Err(Error::Internal(format!(
                "Unsupported conversion: {:?} to {:?}",
                file.content_type(),
                target_type
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: mime::Mime,
    pub content: ProcessedContent,
}

#[derive(Debug, Clone)]
pub enum ProcessedContent {
    Text(String),
    Json(serde_json::Value),
    Image {
        text: String,
        width: u32,
        height: u32,
    },
}

impl ProcessedFile {
    pub fn content_type(&self) -> &'static str {
        match self.content {
            ProcessedContent::Text(_) => "text",
            ProcessedContent::Json(_) => "json",
            ProcessedContent::Image { .. } => "image",
        }
    }

    pub fn raw_data(&self) -> &[u8] {
        // This is a placeholder - in a real implementation,
        // we would store the raw data alongside the processed content
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn processor() -> FileProcessor {
        FileProcessor::new().unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_process_pdf(processor: FileProcessor) -> Result<()> {
        let pdf_data = b"%PDF-1.4\nTest content";
        let path = Path::new("test.pdf");
        
        let processed = processor.process_file(pdf_data, path).await?;
        assert_eq!(processed.content_type(), "text");
        
        if let ProcessedContent::Text(text) = &processed.content {
            assert!(text.contains("Test content"));
        } else {
            panic!("Expected text content");
        }
        
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_process_image(processor: FileProcessor) -> Result<()> {
        let image_data = [0x89, 0x50, 0x4E, 0x47]; // PNG header
        let path = Path::new("test.png");
        
        let processed = processor.process_file(&image_data, path).await?;
        assert_eq!(processed.content_type(), "image");
        
        if let ProcessedContent::Image { width, height, .. } = processed.content {
            assert!(width > 0);
            assert!(height > 0);
        } else {
            panic!("Expected image content");
        }
        
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_convert_file(processor: FileProcessor) -> Result<()> {
        let text = "Test conversion";
        let processed = ProcessedFile {
            id: Uuid::new_v4(),
            original_name: "test.txt".to_string(),
            mime_type: mime::TEXT_PLAIN,
            content: ProcessedContent::Text(text.to_string()),
        };

        let pdf_data = processor.convert_file(&processed, FileType::Pdf).await?;
        assert!(!pdf_data.is_empty());
        
        let word_data = processor.convert_file(&processed, FileType::Word).await?;
        assert!(!word_data.is_empty());
        
        Ok(())
    }
}
