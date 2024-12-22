use gb_core::{Result, Error};
use docx_rs::{Docx, Paragraph, Run, RunText};
use std::io::{Cursor, Read};
use tracing::{instrument, error};

pub struct WordProcessor;

impl WordProcessor {
    #[instrument(skip(data))]
    pub fn extract_text(data: &[u8]) -> Result<String> {
        let doc = Docx::from_reader(Cursor::new(data))
            .map_err(|e| Error::Internal(format!("Failed to read DOCX: {}", e)))?;

        let mut text = String::new();
        for para in doc.document.paragraphs() {
            for run in para.runs() {
                if let Some(text_content) = run.text() {
                    text.push_str(text_content);
                }
                text.push(' ');
            }
            text.push('\n');
        }

        Ok(text)
    }

    #[instrument(skip(content))]
    pub fn create_document(content: &str) -> Result<Vec<u8>> {
        let mut docx = Docx::new();
        
        for line in content.lines() {
            let paragraph = Paragraph::new()
                .add_run(
                    Run::new().add_text(RunText::new(line))
                );
            docx = docx.add_paragraph(paragraph);
        }

        let mut output = Vec::new();
        docx.build()
            .pack(&mut Cursor::new(&mut output))
            .map_err(|e| Error::Internal(format!("Failed to create DOCX: {}", e)))?;

        Ok(output)
    }

    #[instrument(skip(template_data, variables))]
    pub fn fill_template(template_data: &[u8], variables: &serde_json::Value) -> Result<Vec<u8>> {
        let doc = Docx::from_reader(Cursor::new(template_data))
            .map_err(|e| Error::Internal(format!("Failed to read template: {}", e)))?;

        let mut new_doc = doc.clone();
        
        for para in new_doc.document.paragraphs_mut() {
            for run in para.runs_mut() {
                if let Some(text) = run.text_mut() {
                    let mut new_text = text.clone();
                    for (key, value) in variables.as_object().unwrap() {
                        let placeholder = format!("{{{}}}", key);
                        new_text = new_text.replace(&placeholder, value.as_str().unwrap_or(""));
                    }
                    *text = new_text;
                }
            }
        }

        let mut output = Vec::new();
        new_doc.build()
            .pack(&mut Cursor::new(&mut output))
            .map_err(|e| Error::Internal(format!("Failed to save filled template: {}", e)))?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde_json::json;

    #[rstest]
    fn test_create_document() -> Result<()> {
        let content = "Test document\nSecond line";
        let doc_data = WordProcessor::create_document(content)?;
        
        let extracted_text = WordProcessor::extract_text(&doc_data)?;
        assert!(extracted_text.contains("Test document"));
        assert!(extracted_text.contains("Second line"));
        Ok(())
    }

    #[rstest]
    fn test_fill_template() -> Result<()> {
        let template = WordProcessor::create_document("Hello, {name}!")?;
            "name": "World"
        });

        let filled = WordProcessor::fill_template(&template, &variables)?;
        let text = WordProcessor::extract_text(&filled)?;
        assert!(text.contains("Hello, World!"));
        Ok(())
    }
}
