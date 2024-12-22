use gb_core::{Result, Error};
use mime_guess::{from_path, mime};
use std::path::Path;
use tracing::instrument;

pub struct FileTypeDetector;

impl FileTypeDetector {
    #[instrument]
    pub fn detect_mime_type(path: &Path) -> Result<mime::Mime> {
        from_path(path)
            .first_or_octet_stream()
            .to_owned()
            .into()
    }

    #[instrument(skip(data))]
    pub fn detect_from_bytes(data: &[u8]) -> Result<FileType> {
        if data.starts_with(b"%PDF") {
            Ok(FileType::Pdf)
        } else if data.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            // ZIP header, could be DOCX/XLSX
            if Self::is_office_document(data) {
                Ok(FileType::Word)
            } else {
                Ok(FileType::Excel)
            }
        } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            Ok(FileType::Png)
        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Ok(FileType::Jpeg)
        } else if data.starts_with(b"RIFF") && data[8..12] == *b"WEBP" {
            Ok(FileType::WebP)
        } else {
            Err(Error::Internal("Unknown file type".to_string()))
        }
    }

    fn is_office_document(data: &[u8]) -> bool {
        // Check for Office Open XML signatures
        // This is a simplified check
        std::str::from_utf8(data)
            .map(|s| s.contains("word/"))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Pdf,
    Word,
    Excel,
    Png,
    Jpeg,
    WebP,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::path::PathBuf;

    #[rstest]
    fn test_detect_mime_type() -> Result<()> {
        let path = PathBuf::from("test.pdf");
        let mime = FileTypeDetector::detect_mime_type(&path)?;
        assert_eq!(mime.type_(), "application");
        assert_eq!(mime.subtype(), "pdf");
        Ok(())
    }

    #[rstest]
    fn test_detect_from_bytes() -> Result<()> {
        let pdf_data = b"%PDF-1.4\n";
        assert_eq!(FileTypeDetector::detect_from_bytes(pdf_data)?, FileType::Pdf);

        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(FileTypeDetector::detect_from_bytes(&png_data)?, FileType::Png);
        
        Ok(())
    }
}
