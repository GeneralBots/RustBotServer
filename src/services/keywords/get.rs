use rhai::{Dynamic, Engine};
use reqwest;
use crate::services::state::AppState;
use std::error::Error;

pub fn get_keyword(_state: &AppState, engine: &mut Engine) {
    engine.register_custom_syntax(
        &["GET", "$expr$"],
        false, // Expression, not statement
        move |context, inputs| {
            let url = context.eval_expression_tree(&inputs[0])?;
            let url_str = url.to_string();

            if url_str.starts_with("https") {
                println!("HTTPS GET request: {}", url_str);
                
                // Use the same pattern as find_keyword
                let fut = execute_get(&url_str);
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(fut)
                }).map_err(|e| format!("HTTP request failed: {}", e))?;
                
                Ok(Dynamic::from(result))
            } else {
                println!("GET executed: {}", url_str);
                Ok(Dynamic::from(format!("Content from {}", url_str)))
            }
        }
    ).unwrap();
}

pub async fn execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    println!("Starting execute_get with URL: {}", url);
    
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    
    println!("GET request successful, got {} bytes", content.len());
    Ok(format!("Secure content fetched: {}", content))
}