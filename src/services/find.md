- 100KB Email box

i need a vb to russt watx 
first wkeywords;
the keyword find for example will call the websiervics speciifid.
you now ? this should be compatible with languageserer so can be debuger
in code. user rust minamalistc to do this very inline and minimize code verbosity
it will be integatd in antoher proejt so i nee a sevicelayer
to call it from a schduler i need the compiler and the rnner of the ob
use localhost:5858 to call websiervis from the wasm executable
this is will attend 

	FOR EACH item in ARRAY
	next item	
	
	FIND filter
		
	json = FIND "tablename", "field=value"
		/tables/find
		
	SET "tablename", "key=value", "value"
		/tables/set
		
	text = GET "domain.com"
		/webauto/get_text
	
	text = GET WEBSITE "CASAS BAHIA" 'casasbahia.com via duckduckgo.
		/webauto/get_website
	
	CREATE SITE "sitename",company, website, template, prompt 
	
	/sites/create 
		copy template from temapltes folder 
		add llm outpt page.tsx 
		add listener
		
	CREATE DRAFT to, subject, body
	/email/create_draft		








    

use serde_json::{json, Value};
use sqlx::{postgres::PgRow, PgPool, Row};
use sqlx::Column;  // Required for .name() method
use sqlx::TypeInfo;  // Required for .type_info() method
use std::error::Error;
use sqlx::postgres::PgPoolOptions;

// Main async function to execute the FIND command
pub async fn execute_find(
    pool: &PgPool,
    table_str: &str,
    filter_str: &str,
) -> Result<Value, Box<dyn Error>> {
    // Parse the filter string into SQL WHERE clause and parameters
    let (where_clause, params) = parse_filter(filter_str)?;
    
    // Build the SQL query with proper parameter binding
    let query = format!("SELECT * FROM {} WHERE {}", table_str, where_clause);
    
    // Execute query and collect results
    let rows = match params.len() {
        0 => sqlx::query(&query).fetch_all(pool).await?,
        1 => sqlx::query(&query).bind(&params[0]).fetch_all(pool).await?,
        _ => return Err("Only single parameter filters supported in this example".into()),
    };
    
    // Convert rows to JSON values
    let mut results = Vec::new();
    for row in rows {
        results.push(row_to_json(row)?);
    }
    
    // Return the structured result
    Ok(json!({
        "command": "find",
        "table": table_str,
        "filter": filter_str,
        "results": results
    }))
}

fn row_to_json(row: PgRow) -> Result<Value, Box<dyn Error>> {
    let mut result = serde_json::Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let column_name = column.name();
        let value: Value = match column.type_info().name() {
            "int4" | "int8" => {
                match row.try_get::<i64, _>(i) {
                    Ok(v) => json!(v),
                    Err(_) => Value::Null,
                }
            },
            "float4" | "float8" => {
                match row.try_get::<f64, _>(i) {
                    Ok(v) => json!(v),
                    Err(_) => Value::Null,
                }
            },
            "text" | "varchar" => {
                match row.try_get::<String, _>(i) {
                    Ok(v) => json!(v),
                    Err(_) => Value::Null,
                }
            },
            "bool" => {
                match row.try_get::<bool, _>(i) {
                    Ok(v) => json!(v),
                    Err(_) => Value::Null,
                }
            },
            "json" | "jsonb" => {
                match row.try_get::<Value, _>(i) {
                    Ok(v) => v,
                    Err(_) => Value::Null,
                }
            },
            _ => Value::Null,
        };
        result.insert(column_name.to_string(), value);
    }
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
    if !column.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid column name in filter".into());
    }
    
    // Return the parameterized query part and the value separately
    Ok((format!("{} = $1", column), vec![value.to_string()]))
}

// Database connection setup
pub async fn create_pool(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    // Test the connection
    sqlx::query("SELECT 1").execute(&pool).await?;
    
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use dotenv::dotenv;
    use std::env;

    async fn setup_test_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env for tests");
        
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap();

        // Create a test table
        sqlx::query(
            r#"
            DROP TABLE IF EXISTS rob;
            CREATE TABLE rob (
                id SERIAL PRIMARY KEY,
                action TEXT,
                name TEXT,
                is_active BOOLEAN,
                metadata JSONB
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert test data
        sqlx::query(
            r#"
            INSERT INTO rob (action, name, is_active, metadata) VALUES 
            ('EMUL1', 'Robot1', true, '{"version": 1}'),
            ('EMUL2', 'Robot2', false, '{"version": 2}'),
            ('EMUL1', 'Robot3', true, null)
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_execute_find() {
        let pool = setup_test_db().await;
        let result = execute_find(&pool, "rob", "action=EMUL1")
            .await
            .unwrap();
        
        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["action"], "EMUL1");
        assert_eq!(results[1]["action"], "EMUL1");
        
        // Test JSON field
        assert_eq!(results[0]["metadata"]["version"], 1);
        
        // Test boolean field
        assert_eq!(results[0]["is_active"], true);
    }
}