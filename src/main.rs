use std::sync::Arc;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

use services::config::*;
use services::email::*;
use services::file::*;
use services::llm::*;
use services::script::*;
use services::state::*;
use sqlx::PgPool;

use crate::services::web_automation::BrowserPool;
//use services:: find::*;
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
    let app_state = web::Data::new(AppState {
        db: db.into(),
        db_custom: db_custom.into(),
        config: Some(config.clone()),
        minio_client: minio_client.into(),
        browser_pool: browser_pool.clone(),
    });

    let script_service = ScriptService::new(&app_state.clone());

    const TEXT: &str = include_str!("prompts/business/data-enrichment.bas");

    match script_service.compile(TEXT) {
        Ok(ast) => match script_service.run(&ast) {
            Ok(result) => println!("Script executed successfully: {:?}", result),
            Err(e) => eprintln!("Error executing script: {}", e),
        },
        Err(e) => eprintln!("Error compiling script: {}", e),
    }

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
            .service(chat)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
