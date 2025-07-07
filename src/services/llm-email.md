use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
struct ChatRequest {
    input: String,
    context: Option<AppContext>,
}

#[derive(serde::Deserialize)]
struct AppContext {
    view_type: Option<String>,
    email_context: Option<EmailContext>,
}

#[derive(serde::Deserialize)]
struct EmailContext {
    id: String,
    subject: String,
    labels: Vec<String>,
    from: Option<String>,
    to: Option<Vec<String>>,
    body: Option<String>,
}

#[derive(serde::Serialize)]
struct ChatResponse {
    response: String,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(serde::Serialize)]
struct ToolCall {
    tool_name: String,
    parameters: serde_json::Value,
}

#[derive(serde::Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[actix_web::post("/chat")]
pub async fn chat(
    web::Json(request): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let azure_config = from_config(&state.config.clone().unwrap().ai);
    let open_ai = OpenAI::new(azure_config);
    
    // Define available tools based on context
    let tools = get_available_tools(&request.context);
    
    // Build the prompt with context and available tools
    let system_prompt = build_system_prompt(&request.context, &tools);
    let user_message = format!("{}\n\nUser input: {}", system_prompt, request.input);
    
    let response = match open_ai.invoke(&user_message).await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error invoking API: {}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to invoke OpenAI API",
            ));
        }
    };

    // Parse the response for tool calls
    let tool_calls = parse_tool_calls(&response);

    let chat_response = ChatResponse {
        response,
        tool_calls,
    };

    Ok(HttpResponse::Ok().json(chat_response))
}

fn get_available_tools(context: &Option<AppContext>) -> Vec<ToolDefinition> {
    let mut tools = Vec::new();
    
    if let Some(ctx) = context {
        if let Some(view_type) = &ctx.view_type {
            match view_type.as_str() {
                "email" => {
                    tools.push(ToolDefinition {
                        name: "replyEmail".to_string(),
                        description: "Reply to the current email with generated content".to_string(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "The reply content to send"
                                }
                            },
                            "required": ["content"]
                        }),
                    });
                    
                    tools.push(ToolDefinition {
                        name: "forwardEmail".to_string(),
                        description: "Forward the current email to specified recipients".to_string(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "recipients": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Email addresses to forward to"
                                },
                                "content": {
                                    "type": "string",
                                    "description": "Additional message to include"
                                }
                            },
                            "required": ["recipients"]
                        }),
                    });
                }
                _ => {}
            }
        }
    }
    
    tools
}

fn build_system_prompt(context: &Option<AppContext>, tools: &[ToolDefinition]) -> String {
    let mut prompt = String::new();
    
    if let Some(ctx) = context {
        if let Some(view_type) = &ctx.view_type {
            match view_type.as_str() {
                "email" => {
                    if let Some(email_ctx) = &ctx.email_context {
                        prompt.push_str(&format!(
                            "You are an email assistant. Current email context:\n\
                            Subject: {}\n\
                            ID: {}\n\
                            Labels: {:?}\n\n",
                            email_ctx.subject, email_ctx.id, email_ctx.labels
                        ));
                        
                        if let Some(from) = &email_ctx.from {
                            prompt.push_str(&format!("From: {}\n", from));
                        }
                        
                        if let Some(body) = &email_ctx.body {
                            prompt.push_str(&format!("Body: {}\n", body));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    if !tools.is_empty() {
        prompt.push_str("\nAvailable tools:\n");
        for tool in tools {
            prompt.push_str(&format!(
                "- {}: {}\n  Parameters: {}\n\n",
                tool.name, tool.description, tool.parameters
            ));
        }
        
        prompt.push_str(
            "If you need to use a tool, respond with:\n\
            TOOL_CALL: tool_name\n\
            PARAMETERS: {json_parameters}\n\
            RESPONSE: your_response_text\n\n\
            Otherwise, just provide a normal response.\n"
        );
    }
    
    prompt
}

fn parse_tool_calls(response: &str) -> Option<Vec<ToolCall>> {
    if !response.contains("TOOL_CALL:") {
        return None;
    }
    
    let mut tool_calls = Vec::new();
    let lines: Vec<&str> = response.lines().collect();
    
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("TOOL_CALL:") {
            let tool_name = lines[i].replace("TOOL_CALL:", "").trim().to_string();
            
            // Look for parameters in the next line
            if i + 1 < lines.len() && lines[i + 1].starts_with("PARAMETERS:") {
                let params_str = lines[i + 1].replace("PARAMETERS:", "").trim();
                
                if let Ok(parameters) = serde_json::from_str::<serde_json::Value>(params_str) {
                    tool_calls.push(ToolCall {
                        tool_name,
                        parameters,
                    });
                }
            }
        }
        i += 1;
    }
    
    if tool_calls.is_empty() {
        None
    } else {
        Some(tool_calls)
    }
}