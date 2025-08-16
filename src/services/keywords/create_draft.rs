use crate::services::email::save_email_draft;
use crate::services::email::{fetch_latest_sent_to, SaveDraftRequest};
use crate::services::state::AppState;
use rhai::Dynamic;
use rhai::Engine;

pub fn create_draft_keyword(state: &AppState, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["CREATE_DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
            true, // Statement
            move |context, inputs| {
                // Extract arguments
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let reply_text = context.eval_expression_tree(&inputs[2])?.to_string();

                // Execute async operations using the same pattern as FIND
                let fut = execute_create_draft(&state_clone, &to, &subject, &reply_text);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Draft creation error: {}", e))?;

                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

async fn execute_create_draft(
    state: &AppState,
    to: &str,
    subject: &str,
    reply_text: &str,
) -> Result<String, String> {
    let get_result = fetch_latest_sent_to(&state.config.clone().unwrap().email, to).await;
    let email_body = if let Ok(get_result_str) = get_result {
        if !get_result_str.is_empty() {
            let email_separator = "\n\n-------------------------------------------------\n\n"; // Horizontal rule style separator
            reply_text.to_string() + email_separator + get_result_str.as_str()
        } else {
            // Fixed: Use reply_text when get_result_str is empty, not empty string
            reply_text.to_string()
        }
    } else {
        reply_text.to_string()
    };

    // Create and save draft
    let draft_request = SaveDraftRequest {
        to: to.to_string(),
        subject: subject.to_string(),
        cc: None,
        text: email_body,
    };

    let save_result =
        match save_email_draft(&state.config.clone().unwrap().email, &draft_request).await {
            Ok(_) => Ok("Draft saved successfully".to_string()),
            Err(e) => Err(e.to_string()),
        };
    save_result
}
