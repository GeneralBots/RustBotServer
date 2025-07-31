use rhai::Dynamic;
use rhai::Engine;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::services::state::AppState;
use crate::services::utils;

pub fn create_site_keyword(state: &AppState, engine: &mut Engine) {
    let state_clone = state.clone();
    engine
        .register_custom_syntax(
            &[
                "CREATE", "SITE", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$", ",",
                "$expr$",
            ],
            true, // Statement
            move |context, inputs| {
                if inputs.len() < 5 {
                    return Err("Not enough arguments for CREATE SITE".into());
                }

                let _name = context.eval_expression_tree(&inputs[0])?;

                let _website = context.eval_expression_tree(&inputs[2])?;
                let _template = context.eval_expression_tree(&inputs[3])?;
                let prompt = context.eval_expression_tree(&inputs[4])?;
                let ai_config = state_clone.config.as_ref().expect("Config must be initialized").ai.clone();
                // Use the same pattern as find_keyword
                let fut = create_site(&ai_config, _name, prompt);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("HTTP request failed: {}", e))?;

                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

async fn create_site(
    ai_config: &crate::services::config::AIConfig,
    _name: Dynamic,
    prompt: Dynamic,
)  -> Result<String, Box<dyn Error + Send + Sync>> {

    // Call the LLM to generate the HTML contents
    let llm_result = utils::call_llm(&prompt.to_string(), &ai_config).await?;

    // Create the directory structure
    let base_path = "/opt/gbo/tenants/pragmatismo/proxy/data/websites/sites.pragmatismo.com.br";
    let site_name = format!("{}", _name.to_string());
    let full_path = format!("{}/{}", base_path, site_name);

    // Create directory if it doesn't exist
    fs::create_dir_all(&full_path).map_err(|e| e.to_string())?;

    // Write the HTML file
    let index_path = Path::new(&full_path).join("index.html");
    fs::write(index_path, llm_result).map_err(|e| e.to_string())?;

    println!("Site created at: {}", full_path);
    Ok(full_path)
}
