use actix_web::{middleware, web, App, HttpServer};
use gb_core::models;
use tracing_subscriber::fmt::format::FmtSpan;
use dotenv::dotenv;
use gb_core::config::AppConfig;
use gb_core::db::{init_kafka, init_minio, init_postgres, init_redis};
use gb_file::handlers::upload_file;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Load configuration
    let config = AppConfig::from_env();

    // Initialize databases and services
    let db_pool = init_postgres(&config).await.expect("Failed to connect to PostgreSQL");
    let redis_pool = init_redis(&config).await.expect("Failed to connect to Redis");
    let kafka_producer = init_kafka(&config).await.expect("Failed to initialize Kafka");
    // let zitadel_client = init_zitadel(&config).await.expect("Failed to initialize Zitadel");
    let minio_client = init_minio(&config).await.expect("Failed to initialize Minio");

    let app_state = web::Data::new(models::AppState {
        config: config.clone(),
        db_pool,
        redis_pool,
        kafka_producer,
        minio_client,
    });

    // Start HTTP server
    HttpServer::new(move || {
        // let cors = Cors::default()
        //     .allow_any_origin()
        //     .allow_any_method()
        //     .allow_any_header()
        //     .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
//            .wrap(cors)
            .app_data(app_state.clone())
            .service(upload_file)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
