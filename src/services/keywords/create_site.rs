use rhai::Dynamic;
use rhai::Engine;
use std::fs;
use std::path::Path;

use crate::services::state::AppState;

pub fn create_site_keyword(_state: &AppState, engine: &mut Engine) {
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

                // Call the LLM to generate the HTML content
                let llm_result = context.call_fn::<String>("chat", (prompt.to_string(),))?;
                
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
                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}