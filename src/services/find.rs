use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::Column; // Required for .name() method
use sqlx::TypeInfo; // Required for .type_info() method
use sqlx::{postgres::PgRow, PgPool, Row};
use std::error::Error;
use std::time::Duration;


pub async fn execute_find(
    pool: &PgPool,
    table_str: &str,
    filter_str: &str,
) -> Result<Value, String> {  // Changed to String error like your Actix code
    println!("Starting execute_find with table: {}, filter: {}", table_str, filter_str);
    
    let (where_clause, params) = parse_filter(filter_str)
        .map_err(|e| e.to_string())?;
    
    let query = format!("SELECT * FROM {} WHERE {} LIMIT 10", table_str, where_clause);
    println!("Executing query: {}", query);
    
    // Use the same simple pattern as your Actix code - no timeout wrapper
    let rows = sqlx::query(&query)
        .bind(&params[0])  // Simplified like your working code
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("SQL execution error: {}", e);
            e.to_string()
        })?;
    
    println!("Query successful, got {} rows", rows.len());
    
    let mut results = Vec::new();
    for row in rows {
        results.push(row_to_json(row).map_err(|e| e.to_string())?);
    }

    Ok(json!({
        "command": "find",
        "table": table_str,
        "filter": filter_str,
        "results": results
    }))
}

fn row_to_json(row: PgRow) -> Result<Value, Box<dyn Error>> {
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

// Helper function to parse the filter string into SQL WHERE clause and parameters
fn parse_filter(filter_str: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let parts: Vec<&str> = filter_str.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid filter format. Expected 'KEY=VALUE'".into());
    }

    let column = parts[0].trim();
    let value = parts[1].trim();

    // Validate column name to prevent SQL injection
    if !column
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Invalid column name in filter".into());
    }

    // Return the parameterized query part and the value separately
    Ok((format!("{} = $1", column), vec![value.to_string()]))
}
