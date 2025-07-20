use smartstring::SmartString;
use rhai::{Array, Dynamic};
use serde_json::{json, Value};
use sqlx::Column; // Required for .name() method
use sqlx::TypeInfo; // Required for .type_info() method
use sqlx::{postgres::PgRow, Row};

use std::error::Error;

pub fn row_to_json(row: PgRow) -> Result<Value, Box<dyn Error>> {
    let mut result = serde_json::Map::new();
    let columns = row.columns();
    println!("Processing {} columns", columns.len());

    for (i, column) in columns.iter().enumerate() {
        let column_name = column.name();
        let type_name = column.type_info().name();
        println!(
            "Processing column {}: {} (type: {})",
            i, column_name, type_name
        );

        let value: Value = match type_name {
            "INT4" | "INT8" | "int4" | "int8" => match row.try_get::<i64, _>(i) {
                Ok(v) => {
                    println!("Got int64 value: {}", v);
                    json!(v)
                }
                Err(e) => {
                    println!("Failed to get int64, trying i32: {}", e);
                    match row.try_get::<i32, _>(i) {
                        Ok(v) => json!(v as i64),
                        Err(_) => Value::Null,
                    }
                }
            },
            "FLOAT4" | "FLOAT8" | "float4" | "float8" => match row.try_get::<f64, _>(i) {
                Ok(v) => {
                    println!("Got float64 value: {}", v);
                    json!(v)
                }
                Err(e) => {
                    println!("Failed to get float64, trying f32: {}", e);
                    match row.try_get::<f32, _>(i) {
                        Ok(v) => json!(v as f64),
                        Err(_) => Value::Null,
                    }
                }
            },
            "TEXT" | "VARCHAR" | "text" | "varchar" => match row.try_get::<String, _>(i) {
                Ok(v) => {
                    println!("Got string value: {}", v);
                    json!(v)
                }
                Err(e) => {
                    println!("Failed to get string: {}", e);
                    Value::Null
                }
            },
            "BOOL" | "bool" => match row.try_get::<bool, _>(i) {
                Ok(v) => {
                    println!("Got bool value: {}", v);
                    json!(v)
                }
                Err(e) => {
                    println!("Failed to get bool: {}", e);
                    Value::Null
                }
            },
            "JSON" | "JSONB" | "json" | "jsonb" => match row.try_get::<Value, _>(i) {
                Ok(v) => {
                    println!("Got JSON value: {:?}", v);
                    v
                }
                Err(e) => {
                    println!("Failed to get JSON, trying as string: {}", e);
                    match row.try_get::<String, _>(i) {
                        Ok(s) => match serde_json::from_str(&s) {
                            Ok(v) => v,
                            Err(_) => json!(s),
                        },
                        Err(_) => Value::Null,
                    }
                }
            },
            _ => {
                println!("Unknown type {}, trying as string", type_name);
                match row.try_get::<String, _>(i) {
                    Ok(v) => json!(v),
                    Err(_) => Value::Null,
                }
            }
        };
        result.insert(column_name.to_string(), value);
    }
    println!("Finished processing row, got {} fields", result.len());
    Ok(Value::Object(result))
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
