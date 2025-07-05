use actix_web::http::Error;
use actix_web::{
    get, post,
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use futures::StreamExt;
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    fmt_message, fmt_template,
    language_models::llm::LLM,
    llm::{openai::OpenAI, AzureConfig},
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::messages::Message,
    template_fstring,
};
use std::env;

use crate::services::{config::AIConfig, state::AppState};

pub fn from_config(config: &AIConfig) -> AzureConfig {
    AzureConfig::default()
        .with_api_key(&config.key)
        .with_api_base(&config.endpoint)
        .with_api_version(&config.version)
        .with_deployment_id(&config.instance)
}

#[derive(serde::Deserialize)]
struct ChatRequest {
    input: String,
}#[actix_web::post("/chat")]
pub async fn chat(
    web::Json(request): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let azure_config = from_config(&state.config.clone().unwrap().ai);
    let open_ai = OpenAI::new(azure_config);

    let response = match open_ai.invoke(&request.input).await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error invoking API: {}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to invoke OpenAI API",
            ));
        }
    };
    Ok(HttpResponse::Ok().body(response))
}


#[actix_web::post("/stream")]
pub async fn chat_stream(
    web::Json(request): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let azure_config = from_config(&state.config.clone().unwrap().ai);
    let open_ai = OpenAI::new(azure_config);

    let response = match open_ai.invoke("Why is the sky blue?").await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error invoking API: {}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to invoke OpenAI API",
            ));
        }
    };

    let prompt = message_formatter![
        fmt_message!(Message::new_system_message(
            "You are world class technical documentation writer."
        )),
        fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
            "{input}", "input"
        )))
    ];

    let chain = LLMChainBuilder::new()
        .prompt(prompt)
        .llm(open_ai)
        .build()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut stream = chain
        .stream(prompt_args! { "input" => request.input })
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let actix_stream = async_stream::stream! {
        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => yield Ok::<_, actix_web::Error>(Bytes::from(value.content)),
                Err(e) => yield Err(actix_web::error::ErrorInternalServerError(e)),
            }
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(actix_stream))
}
