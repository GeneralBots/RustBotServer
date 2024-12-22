use gb_core::{Result, Error};
use lopdf::{Document, Object, StringFormat};
use std::io::Cursor;
use tracing::{instrument, error};

pub struct PdfProcessor;

impl PdfProcessor {
    #[instrument(skip(data))]
    pub fn extract_text(data: &[u8]) -> Result<String> {
        let doc = Document::load_from(Cursor::new(data))
            .map_err(|e| Error::Internal(format!("Failed to load PDF: {}", e)))?;

        let mut text = String::new();
        for page_num in 1..=doc.get_pages().len() {
            if let Ok(page_text) = Self::extract_page_text(&doc, page_num) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }

        Ok(text)
    }

    #[instrument(skip(doc))]
    fn extract_page_text(doc: &Document, page_num: u32) -> Result<String> {
        let page = doc.get_page(page_num)
            .map_err(|e| Error::Internal(format!("Failed to get page {}: {}", page_num, e)))?;

        let contents = doc.get_page_content(page)
            .map_err(|e| Error::Internal(format!("Failed to get page content: {}", e)))?;

        let mut text = String::new();
        for content in contents.iter() {
            if let Ok(Object::String(s, StringFormat::Literal)) = content {
                if let Ok(decoded) = String::from_utf8(s.clone()) {
                    text.push_str(&decoded);
                }
            }
        }

        Ok(text)
    }

    #[instrument(skip(data))]
    pub fn merge_pdfs(pdfs: Vec<&[u8]>) -> Result<Vec<u8>> {
        let mut merged = Document::new();
        let mut current_page = 1;

        for pdf_data in pdfs {
            let doc = Document::load_from(Cursor::new(pdf_data))
                .map_err(|e| Error::Internal(format!("Failed to load PDF: {}", e)))?;

            for (_, page) in doc.get_pages() {
                merged.add_page(page.clone());
                current_page += 1;
            }
        }

        let mut output = Vec::new();
        merged.save_to(&mut Cursor::new(&mut output))
            .map_err(|e| Error::Internal(format!("Failed to save merged PDF: {}", e)))?;

        Ok(output)
    }

    #[instrument(skip(data))]
    pub fn split_pdf(data: &[u8], pages: &[u32]) -> Result<Vec<Vec<u8>>> {
        let doc = Document::load_from(Cursor::new(data))
            .map_err(|e| Error::Internal(format!("Failed to load PDF: {}", e)))?;

        let mut result = Vec::new();
        for &page_num in pages {
            let mut new_doc = Document::new();
            if let Ok(page) = doc.get_page(page_num) {
                new_doc.add_page(page.clone());
                let mut output = Vec::new();
                new_doc.save_to(&mut Cursor::new(&mut output))
                    .map_err(|e| Error::Internal(format!("Failed to save split PDF: {}", e)))?;
                result.push(output);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    fn create_test_pdf() -> Vec<u8> {
        let mut doc = Document::new();
        doc.add_page(lopdf::dictionary! {
            "Type" => "Page",
            "Contents" => Object::String(b"BT /F1 12 Tf 72 712 Td (Test Page) Tj ET".to_vec(), StringFormat::Literal),
        });
        let mut output = Vec::new();
        doc.save_to(&mut Cursor::new(&mut output)).unwrap();
        output
    }

    #[rstest]
    fn test_extract_text() -> Result<()> {
        let pdf_data = create_test_pdf();
        let text = PdfProcessor::extract_text(&pdf_data)?;
        assert!(text.contains("Test Page"));
        Ok(())
    }

    #[rstest]
    fn test_merge_pdfs() -> Result<()> {
        let pdf1 = create_test_pdf();
        let pdf2 = create_test_pdf();
        let merged = PdfProcessor::merge_pdfs(vec[build-dependencies]&pdf1, &pdf2])?;
        Ok(())
    }

    #[rstest]
    fn test_split_pdf() -> Result<()> {
        let pdf_data = create_test_pdf();
        let split = PdfProcessor::split_pdf(&pdf_data, &[1])?;
        assert_eq!(split.len(), 1);
        Ok(())
    }
}
