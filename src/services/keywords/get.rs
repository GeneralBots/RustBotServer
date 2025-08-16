use crate::services::state::AppState;
use reqwest::{self, Client};
use rhai::{Dynamic, Engine};
use scraper::{Html, Selector};
use std::error::Error;

pub fn get_keyword(_state: &AppState, engine: &mut Engine) {
    engine
        .register_custom_syntax(
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
                    })
                    .map_err(|e| format!("HTTP request failed: {}", e))?;

                    Ok(Dynamic::from(result))
                } else {
                    println!("GET executed: {}", url_str);
                    Ok(Dynamic::from(format!("Content from {}", url_str)))
                }
            },
        )
        .unwrap();
}

pub async fn _execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    println!("Starting execute_get with URL: {}", url);

    let response = reqwest::get(url).await?;
    let content = response.text().await?;

    println!("GET request successful, got {} bytes", content.len());
    Ok(format!("Secure content fetched: {}", content))
}

pub async fn execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    println!("Starting execute_get with URL: {}", url);

    // Create a client that ignores invalid certificates
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client.get(url).send().await?;
    let html_content = response.text().await?;

    // Parse HTML and extract text
    let document = Html::parse_document(&html_content);
    let selector = Selector::parse("body").unwrap(); // Focus on body content
    let body = document.select(&selector).next().unwrap();
    let text_content = body.text().collect::<Vec<_>>().join(" ");

    // Clean up the text (remove extra whitespace, newlines, etc.)
    let cleaned_text = text_content
        .replace('\n', " ")
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        "GET request successful, extracted {} characters of text",
        cleaned_text.len()
    );
    Ok(cleaned_text)
}
