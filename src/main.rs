use actix_web::{middleware, web, App, HttpServer};
use tracing_subscriber::fmt::format::FmtSpan;
use dotenv::dotenv;

use services::config::*;
use services::file::*;
use services::state::*;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // log::set_max_level(LevelFilter::Info);

    let config = AppConfig::from_env();
    let minio_client = init_minio(&config).await.expect("Failed to initialize Minio");

    let app_state = web::Data::new(AppState {
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

        })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
