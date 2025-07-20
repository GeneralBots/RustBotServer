use rhai::Dynamic;
use rhai::Engine;
use serde_json::json;

use crate::services::state::AppState;

pub fn create_site_keyword(_state: &AppState, engine: &mut Engine) {

    engine
        .register_custom_syntax(
            &[
                "CREATE", "SITE", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$", ",",
                "$expr$",
            ],
            true, // Statement
            |context, inputs| {
                if inputs.len() < 5 {
                    return Err("Not enough arguments for CREATE SITE".into());
                }

                let name = context.eval_expression_tree(&inputs[0])?;
                let company = context.eval_expression_tree(&inputs[1])?;
                let website = context.eval_expression_tree(&inputs[2])?;
                let template = context.eval_expression_tree(&inputs[3])?;
                let prompt = context.eval_expression_tree(&inputs[4])?;

                let result = json!({
                    "command": "create_site",
                    "name": name.to_string(),
                    "company": company.to_string(),
                    "website": website.to_string(),
                    "template": template.to_string(),
                    "prompt": prompt.to_string()
                });
                println!("CREATE SITE executed: {}", result.to_string());
                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
