use rhai::Dynamic;
use rhai::Engine;

use crate::services::state::AppState;

pub fn print_keyword(_state: &AppState, engine: &mut Engine) {

    // PRINT command
    engine
        .register_custom_syntax(
            &["PRINT", "$expr$"],
            true, // Statement
            |context, inputs| {
                let value = context.eval_expression_tree(&inputs[0])?;
                println!("{}", value);
                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
