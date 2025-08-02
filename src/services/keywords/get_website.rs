use crate::services::{state::AppState, web_automation::BrowserPool};
use rhai::{Dynamic, Engine};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use thirtyfour::{By, WebDriver};
use tokio::time::sleep;

pub fn get_website_keyword(state: &AppState, engine: &mut Engine) {
    let browser_pool = state.browser_pool.clone(); // Assuming AppState has browser_pool field

    engine
        .register_custom_syntax(
            &["WEBSITE", "OF", "$expr$"],
            false,
            move |context, inputs| {
                let search_term = context.eval_expression_tree(&inputs[0])?.to_string();
                

                println!(
                    "GET WEBSITE executed - Search: '{}'",
                    search_term
                );

                let browser_pool_clone = browser_pool.clone();
                let fut = execute_headless_browser_search(
                    browser_pool_clone,
                    &search_term
                );

                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Headless browser search failed: {}", e))?;

                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

pub async fn execute_headless_browser_search(
    browser_pool: Arc<BrowserPool>, // Adjust path as needed
    search_term: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    println!(
        "Starting headless browser search: '{}' ",
        search_term
    );

    let search_term = search_term.to_string();
    

    let result = browser_pool
        .with_browser(|driver| {
            Box::pin(async move { perform_search(driver, &search_term).await })
        })
        .await?;

    Ok(result)
}
async fn perform_search(
    driver: WebDriver,
    search_term: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Navigate to DuckDuckGo
    driver.goto("https://duckduckgo.com").await?;

    // Wait for search box and type query
    let search_input = driver.find(By::Id("searchbox_input")).await?;
    search_input.click().await?;
    search_input.send_keys(search_term).await?;

    // Submit search by pressing Enter
    search_input.send_keys("\n").await?;

    // Wait for results to load - using a modern result selector
    driver.find(By::Css("[data-testid='result']")).await?;
    sleep(Duration::from_millis(2000)).await;

    // Extract results
    let results = extract_search_results(&driver).await?;
    driver.close_window().await?;

    if !results.is_empty() {
        Ok(results[0].clone())
    } else {
        Ok("No results found".to_string())
    }
}

async fn extract_search_results(
    driver: &WebDriver,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let mut results = Vec::new();

    // Try different selectors for search results, ordered by most specific to most general
    let selectors = [
        // Modern DuckDuckGo (as seen in the HTML)
        "a[data-testid='result-title-a']", // Primary result links
        "a[data-testid='result-extras-url-link']", // URL links in results
        "a.eVNpHGjtxRBq_gLOfGDr", // Class-based selector for result titles
        "a.Rn_JXVtoPVAFyGkcaXyK", // Class-based selector for URL links
        ".ikg2IXiCD14iVX7AdZo1 a", // Heading container links
        ".OQ_6vPwNhCeusNiEDcGp a", // URL container links
        // Fallback selectors
        ".result__a", // Classic DuckDuckGo
        "a.result-link", // Alternative
        ".result a[href]", // Generic result links
    ];

    for selector in &selectors {
        if let Ok(elements) = driver.find_all(By::Css(selector)).await {
            for element in elements {
                if let Ok(Some(href)) = element.attr("href").await {
                    // Filter out internal and non-http links
                    if href.starts_with("http") 
                        && !href.contains("duckduckgo.com")
                        && !href.contains("duck.co")
                        && !results.contains(&href) {
                        
                        // Get the display URL for verification
                        let display_url = if let Ok(text) = element.text().await {
                            text.trim().to_string()
                        } else {
                            String::new()
                        };

                        // Only add if it looks like a real result (not an ad or internal link)
                        if !display_url.is_empty() && !display_url.contains("Ad") {
                            results.push(href);
                        }
                    }
                }
            }
            if !results.is_empty() {
                break;
            }
        }
    }

    // Deduplicate results
    results.dedup();

    Ok(results)
}