use rhai::Dynamic;
use rhai::Engine;

use crate::services::state::AppState;

pub fn get_keyword(_state: &AppState, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            &["GET", "$expr$"],
            false, // Expression, not statement
            |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?;
                let url_str = url.to_string();

                println!("GET executed: {}", url_str.to_string());
                Ok(format!("Content from {}", url_str).into())
            },
        )
        .unwrap();
}
