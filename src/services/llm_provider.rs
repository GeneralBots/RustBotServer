use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

// OpenAI-compatible request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: String,
}

// Proxy endpoint
#[post("/v1/chat/completions")]
async fn chat_completions(
    req_body: web::Json<ChatCompletionRequest>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    dotenv().ok();

    // Environment variables
    let azure_endpoint = env::var("AI_ENDPOINT")
        .map_err(|_| actix_web::error::ErrorInternalServerError("AI_ENDPOINT not set."))?;
    let azure_key = env::var("AI_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("AI_KEY not set."))?;
    let deployment_name = env::var("AI_LLM_MODEL")
        .map_err(|_| actix_web::error::ErrorInternalServerError("AI_LLM_MODEL not set."))?;

    // Construct Azure OpenAI URL
    let url = format!(
        "{}/openai/deployments/{}/chat/completions?api-version=2025-01-01-preview",
        azure_endpoint, deployment_name
    );

    // Forward headers
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "api-key",
        reqwest::header::HeaderValue::from_str(&azure_key)
            .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid Azure key"))?,
    );
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    // Send request to Azure
    let client = Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .json(&req_body.into_inner())
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Handle response based on status
    let status = response.status();
    let raw_response = response
        .text()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log the raw response
    println!("Raw Azure response: {}", raw_response);

    if status.is_success() {
        // Parse the raw response as JSON
        let azure_response: serde_json::Value = serde_json::from_str(&raw_response)
            .map_err(actix_web::error::ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().json(azure_response))
    } else {
        // Handle error responses properly
        let actix_status = actix_web::http::StatusCode::from_u16(status.as_u16())
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        Ok(HttpResponse::build(actix_status).body(raw_response))
    }
}
