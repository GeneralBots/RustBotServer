use rhai::Dynamic;
use rhai::Engine;
use serde_json::{json, Value};
use sqlx::{PgPool};
use std::error::Error;

use crate::services::state::AppState;
use crate::services::utils;


pub fn set_keyword(state: &AppState, engine: &mut Engine) {
    let db = state.db_custom.clone();

    engine
        .register_custom_syntax(&["SET", "$expr$", ",", "$expr$", ",", "$expr$"], false, {
            let db = db.clone();

            move |context, inputs| {
                let table_name = context.eval_expression_tree(&inputs[0])?;
                let filter = context.eval_expression_tree(&inputs[1])?;
                let updates = context.eval_expression_tree(&inputs[2])?;
                let binding = db.as_ref().unwrap();

                // Use the current async context instead of creating a new runtime
                let binding2 = table_name.to_string();
                let binding3 = filter.to_string();
                let binding4 = updates.to_string();
                let fut = execute_set(binding, &binding2, &binding3, &binding4);

                // Use tokio::task::block_in_place + tokio::runtime::Handle::current().block_on
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("No rows affected".into())
                }
            }
        })
        .unwrap();
}

pub async fn execute_set(
    pool: &PgPool,
    table_str: &str,
    filter_str: &str,
    updates_str: &str,
) -> Result<Value, String> {
    println!(
        "Starting execute_set with table: {}, filter: {}, updates: {}",
        table_str, filter_str, updates_str
    );

    // Parse the updates first to know how many parameters we'll have
    let (set_clause, update_params) = parse_updates(updates_str).map_err(|e| e.to_string())?;
    let update_params_count = update_params.len();

    // Parse the filter condition with an offset for parameter indices
    let (where_clause, filter_params) = utils::parse_filter_with_offset(filter_str, update_params_count)
        .map_err(|e| e.to_string())?;

    // Combine all parameters (updates first, then filter)
    let mut params = update_params;
    params.extend(filter_params);

    let query = format!(
        "UPDATE {} SET {} WHERE {}",
        table_str, set_clause, where_clause
    );
    println!("Executing query: {}", query);

    // Execute the update with all parameters
    let mut query_builder = sqlx::query(&query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }

    let result = query_builder
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("SQL execution error: {}", e);
            e.to_string()
        })?;

    println!("Update successful, affected {} rows", result.rows_affected());

    Ok(json!({
        "command": "set",
        "table": table_str,
        "filter": filter_str,
        "updates": updates_str,
        "rows_affected": result.rows_affected()
    }))
}

// Helper function to parse the updates string into SQL SET clause and parameters
fn parse_updates(updates_str: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let mut set_clauses = Vec::new();
    let mut params = Vec::new();
    
    // Split multiple updates by comma
    for (i, update) in updates_str.split(',').enumerate() {
        let parts: Vec<&str> = update.split('=').collect();
        if parts.len() != 2 {
            return Err("Invalid update format. Expected 'KEY=VALUE'".into());
        }

        let column = parts[0].trim();
        let value = parts[1].trim();

        // Validate column name to prevent SQL injection
        if !column.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Invalid column name in update".into());
        }

        set_clauses.push(format!("{} = ${}", column, i + 1));
        params.push(value.to_string());
    }

    Ok((set_clauses.join(", "), params))
}