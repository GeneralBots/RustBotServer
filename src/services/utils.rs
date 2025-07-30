use crate::services::config::AIConfig;
use langchain_rust::llm::OpenAI;
use langchain_rust::{language_models::llm::LLM, llm::AzureConfig};
use log::{debug, warn};
use rhai::{Array, Dynamic};
use serde_json::{json, Value};
use smartstring::SmartString;
use sqlx::Column; // Required for .name() method
use sqlx::TypeInfo; // Required for .type_info() method
use sqlx::{postgres::PgRow, Row};
use sqlx::{Decode, Type};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tokio_stream::StreamExt;
use zip::ZipArchive;
use tokio::fs::File as TokioFile;

use reqwest::Client;
use tokio::io::AsyncWriteExt;

pub fn azure_from_config(config: &AIConfig) -> AzureConfig {
    AzureConfig::default()
        .with_api_key(&config.key)
        .with_api_base(&config.endpoint)
        .with_api_version(&config.version)
        .with_deployment_id(&config.instance)
}

pub async fn call_llm(
    text: &str,
    ai_config: &AIConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let azure_config = azure_from_config(&ai_config.clone());
    let open_ai = OpenAI::new(azure_config);

    // Directly use the input text as prompt
    let prompt = text.to_string();

    // Call LLM and return the raw text response
    match open_ai.invoke(&prompt).await {
        Ok(response_text) => Ok(response_text),
        Err(err) => {
            eprintln!("Error invoking LLM API: {}", err);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to invoke LLM API",
            )))
        }
    }
}

pub fn extract_zip_recursive(
    zip_path: &Path,
    destination_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let buf_reader = BufReader::new(file);
    let mut archive = ZipArchive::new(buf_reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination_path.join(file.mangled_name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(&parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
pub fn row_to_json(row: PgRow) -> Result<Value, Box<dyn Error>> {
    let mut result = serde_json::Map::new();
    let columns = row.columns();
    debug!("Converting row with {} columns", columns.len());

    for (i, column) in columns.iter().enumerate() {
        let column_name = column.name();
        let type_name = column.type_info().name();

        let value = match type_name {
            "INT4" | "int4" => handle_nullable_type::<i32>(&row, i, column_name),
            "INT8" | "int8" => handle_nullable_type::<i64>(&row, i, column_name),
            "FLOAT4" | "float4" => handle_nullable_type::<f32>(&row, i, column_name),
            "FLOAT8" | "float8" => handle_nullable_type::<f64>(&row, i, column_name),
            "TEXT" | "VARCHAR" | "text" | "varchar" => {
                handle_nullable_type::<String>(&row, i, column_name)
            }
            "BOOL" | "bool" => handle_nullable_type::<bool>(&row, i, column_name),
            "JSON" | "JSONB" | "json" | "jsonb" => handle_json(&row, i, column_name),
            _ => {
                warn!("Unknown type {} for column {}", type_name, column_name);
                handle_nullable_type::<String>(&row, i, column_name)
            }
        };

        result.insert(column_name.to_string(), value);
    }

    Ok(Value::Object(result))
}

fn handle_nullable_type<'r, T>(row: &'r PgRow, idx: usize, col_name: &str) -> Value
where
    T: Type<sqlx::Postgres> + Decode<'r, sqlx::Postgres> + serde::Serialize + std::fmt::Debug,
{
    match row.try_get::<Option<T>, _>(idx) {
        Ok(Some(val)) => {
            debug!("Successfully read column {} as {:?}", col_name, val);
            json!(val)
        }
        Ok(None) => {
            debug!("Column {} is NULL", col_name);
            Value::Null
        }
        Err(e) => {
            warn!("Failed to read column {}: {}", col_name, e);
            Value::Null
        }
    }
}

fn handle_json(row: &PgRow, idx: usize, col_name: &str) -> Value {
    // First try to get as Option<Value>
    match row.try_get::<Option<Value>, _>(idx) {
        Ok(Some(val)) => {
            debug!("Successfully read JSON column {} as Value", col_name);
            return val;
        }
        Ok(None) => return Value::Null,
        Err(_) => (), // Fall through to other attempts
    }

    // Try as Option<String> that might contain JSON
    match row.try_get::<Option<String>, _>(idx) {
        Ok(Some(s)) => match serde_json::from_str(&s) {
            Ok(val) => val,
            Err(_) => {
                debug!("Column {} contains string that's not JSON", col_name);
                json!(s)
            }
        },
        Ok(None) => Value::Null,
        Err(e) => {
            warn!("Failed to read JSON column {}: {}", col_name, e);
            Value::Null
        }
    }
}
pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => Dynamic::from(
            arr.iter()
                .map(json_value_to_dynamic)
                .collect::<rhai::Array>(),
        ),
        Value::Object(obj) => Dynamic::from(
            obj.iter()
                .map(|(k, v)| (SmartString::from(k), json_value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

/// Converts any value to an array - single values become single-element arrays
pub fn to_array(value: Dynamic) -> Array {
    if value.is_array() {
        // Already an array - return as-is
        value.cast::<Array>()
    } else if value.is_unit() || value.is::<()>() {
        // Handle empty/unit case
        Array::new()
    } else {
        // Convert single value to single-element array
        Array::from([value])
    }
}

pub async fn download_file(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let mut file = TokioFile::create(output_path).await?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            file.write_all(&chunk?).await?;
        }
        debug!("File downloaded successfully to {}", output_path);
    } else {
        return Err("Failed to download file".into());
    }

    Ok(())
}
