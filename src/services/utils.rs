use log::{debug, warn};
use rhai::{Array, Dynamic};
use serde_json::{json, Value};
use smartstring::SmartString;
use sqlx::Column; // Required for .name() method
use sqlx::TypeInfo; // Required for .type_info() method
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use sqlx::{Decode, Type};

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
            "TEXT" | "VARCHAR" | "text" | "varchar" => handle_nullable_type::<String>(&row, i, column_name),
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
