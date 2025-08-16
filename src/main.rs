use std::sync::Arc;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use services::state::*;
use services::{config::*, file::*};
use sqlx::PgPool;

use crate::services::automation::AutomationService;
use crate::services::email::{get_emails, list_emails, save_click, send_email};
use crate::services::llm::{chat, chat_stream};
use crate::services::llm_provider::chat_completions;
use crate::services::web_automation::{initialize_browser_pool, BrowserPool};

mod models;
mod services;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::from_env();
    let db_url = config.database_url();
    let db_custom_url = config.database_custom_url();
    let db = PgPool::connect(&db_url).await.unwrap();
    let db_custom = PgPool::connect(&db_custom_url).await.unwrap();

    let minio_client = init_minio(&config)
        .await
        .expect("Failed to initialize Minio");

    let browser_pool = Arc::new(BrowserPool::new(
        "http://localhost:9515".to_string(),
        5,
        "/usr/bin/brave-browser-beta".to_string(),
    ));

    #[cfg(feature = "local_llm")]
    {
        use crate::services::llm_local::ensure_llama_server_running;

        ensure_llama_server_running()
            .await
            .expect("Failed to initialize LLM local server.");
    }

    initialize_browser_pool()
        .await
        .expect("Failed to initialize browser pool");

    let app_state = web::Data::new(AppState {
        db: db.into(),
        db_custom: db_custom.into(),
        config: Some(config.clone()),
        minio_client: minio_client.into(),
        browser_pool: browser_pool.clone(),
    });

    // Start automation service in background
    let automation_state = app_state.get_ref().clone(); // This gets the Arc<AppState>

    let automation = AutomationService::new(automation_state, "src/prompts");
    let _automation_handle = automation.spawn();

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(upload_file)
            .service(list_file)
            .service(save_click)
            .service(get_emails)
            .service(list_emails)
            .service(send_email)
            .service(chat_stream)
            .service(chat_completions)
            .service(chat)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
