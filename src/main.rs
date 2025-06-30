use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::PgPool;

use services::config::*;
use services::email::*;
use services::file::*;
use services::state::*;

mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = AppConfig::from_env();

    let db_url = config.database_url(); 
    let db = PgPool::connect(&db_url).await.unwrap();

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
