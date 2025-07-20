use rhai::Dynamic;
use rhai::Engine;
use serde_json::{json, Value};
use sqlx::{PgPool};
use std::error::Error;

use crate::services::state::AppState;
use crate::services::utils;
use crate::services::utils::row_to_json;
use crate::services::utils::to_array;


pub fn find_keyword(state: &AppState, engine: &mut Engine) {
    let db = state.db_custom.clone();

    engine
        .register_custom_syntax(&["FIND", "$expr$", ",", "$expr$"], false, {
            let db = db.clone();

            move |context, inputs| {
                let table_name = context.eval_expression_tree(&inputs[0])?;
                let filter = context.eval_expression_tree(&inputs[1])?;
                let binding = db.as_ref().unwrap();

                // Use the current async context instead of creating a new runtime
                let binding2 = table_name.to_string();
                let binding3 = filter.to_string();
                let fut = execute_find(binding, &binding2, &binding3);

                // Use tokio::task::block_in_place + tokio::runtime::Handle::current().block_on
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("DB error: {}", e))?;

                if let Some(results) = result.get("results") {
                    let array = to_array(utils::json_value_to_dynamic(results));
                    Ok(Dynamic::from(array))
                } else {
                    Err("No results".into())
                }
            }
        })
        .unwrap();
}

pub async fn execute_find(
    pool: &PgPool,
    table_str: &str,
    filter_str: &str,
) -> Result<Value, String> {
    // Changed to String error like your Actix code
    println!(
        "Starting execute_find with table: {}, filter: {}",
        table_str, filter_str
    );

    let (where_clause, params) = parse_filter(filter_str).map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT * FROM {} WHERE {} LIMIT 10",
        table_str, where_clause
    );
    println!("Executing query: {}", query);

    // Use the same simple pattern as your Actix code - no timeout wrapper
    let rows = sqlx::query(&query)
        .bind(&params[0]) // Simplified like your working code
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
