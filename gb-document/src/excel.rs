use gb_core::{Result, Error};
use calamine::{Reader, Xlsx, RangeDeserializerBuilder};
use std::io::Cursor;
use tracing::{instrument, error};

pub struct ExcelProcessor;

impl ExcelProcessor {
    #[instrument(skip(data))]
    pub fn extract_data(data: &[u8]) -> Result<Vec<Vec<String>>> {
        let cursor = Cursor::new(data);
        let mut workbook = Xlsx::new(cursor)
            .map_err(|e| Error::Internal(format!("Failed to read Excel file: {}", e)))?;

        let sheet_name = workbook.sheet_names()[0].clone();
        let range = workbook.worksheet_range(&sheet_name)
            .ok_or_else(|| Error::Internal("Failed to get worksheet".to_string()))?
            .map_err(|e| Error::Internal(format!("Failed to read range: {}", e)))?;

        let mut result = Vec::new();
        for row in range.rows() {
            let row_data: Vec<String> = row.iter()
                .map(|cell| cell.to_string())
                .collect();
            result.push(row_data);
        }

        Ok(result)
    }

    #[instrument(skip(headers, data))]
    pub fn create_excel(headers: &[&str], data: &[Vec<String>]) -> Result<Vec<u8>> {
        todo!("Implement Excel creation using a suitable library");
    }

