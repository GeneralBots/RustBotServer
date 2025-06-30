use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::PgPool;
use tracing_subscriber::fmt::format::FmtSpan;

use services::config::*;
use services::file::*;
use services::state::*;
use services::email::*;


mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv().ok();
    let config = AppConfig::from_env();
    let db = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let minio_client = init_minio(&config)
        .await
        .expect("Failed to initialize Minio");

    let app_state = web::Data::new(AppState {
        db: Some(db.clone()),
        config: Some(config.clone()),
        minio_client: Some(minio_client),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app_state.clone())
            .service(upload_file)
            .service(list_file)
            .service(save_click)
            .service(get_emails)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
