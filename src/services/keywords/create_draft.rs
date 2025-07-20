use rhai::Dynamic;
use rhai::Engine;
use serde_json::json;

use crate::services::state::AppState;

pub fn create_draft_keyword(_state: &AppState, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            &["CREATE", "DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
            true, // Statement
            |context, inputs| {
                if inputs.len() < 3 {
                    return Err("Not enough arguments for CREATE DRAFT".into());
                }

                let to = context.eval_expression_tree(&inputs[0])?;
                let subject = context.eval_expression_tree(&inputs[1])?;
                let body = context.eval_expression_tree(&inputs[2])?;

                let result = json!({
                    "command": "create_draft",
                    "to": to.to_string(),
                    "subject": subject.to_string(),
                    "body": body.to_string()
                });
                println!("CREATE DRAFT executed: {}", result.to_string());
                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
