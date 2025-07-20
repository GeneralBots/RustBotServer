use rhai::Dynamic;
use rhai::Engine;
use serde_json::json;

use crate::services::state::AppState;


pub fn set_keyword(_state: &AppState, engine: &mut Engine) {

    engine
            .register_custom_syntax(
                &["SET", "$expr$", ",", "$expr$", ",", "$expr$"],
                true, // Statement
                |context, inputs| {
                    let table_name = context.eval_expression_tree(&inputs[0])?;
                    let key_value = context.eval_expression_tree(&inputs[1])?;
                    let value = context.eval_expression_tree(&inputs[2])?;

                    let table_str = table_name.to_string();
                    let key_str = key_value.to_string();
                    let value_str = value.to_string();

                    let result = json!({
                        "command": "set",
                        "status": "success",
                        "table": table_str,
                        "key": key_str,
                        "value": value_str
                    });
                    println!("SET executed: {}", result.to_string());
                    Ok(Dynamic::UNIT)
                },
            )
            .unwrap();

}
