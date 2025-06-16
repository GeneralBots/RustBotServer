use log::{info, LevelFilter};


use tokio::net::TcpStream;

use actix_web::{middleware, web, App, HttpServer};
use gb_core::models;
use tracing_subscriber::fmt::format::FmtSpan;
use dotenv::dotenv;
use gb_core::config::AppConfig;
use gb_core::db::{init_minio, init_postgres};
use gb_file::handlers::upload_file;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Configure the logger
    // log::set_logger(&VectorLogger { stream: TcpStream::connect("127.0.0.1:9000").await? })
    //     .map_err(|_| "Couldn't set logger")?;
    // log::set_max_level(LevelFilter::Info);

    // Get the Vector agent's address and port
    let vector_host = "127.0.0.1";
    let vector_port = 9000;

    // // Start a Vector logger
    // let mut vector_logger = VectorLogger::new(vector_host, vector_port).await?;

    // // Set the logger
    // log::set_logger(&vector_logger).map_err(|_| "Couldn't set logger")?;
    // log::set_max_level(LevelFilter::Info);

    // Log some messages
    info!("Hello from Rust!");


    // Load configuration
    let config = AppConfig::from_env();


    // TODO: /gbo/bin/storage$ ./minio server ../../data/storage/



    // Initialize databases and services
    let db_pool = init_postgres(&config).await.expect("Failed to connect to PostgreSQL");
    let minio_client = init_minio(&config).await.expect("Failed to initialize Minio");

    let app_state = web::Data::new(models::AppState {
        config: Some(config.clone()),
        db_pool: Some(db_pool),
        minio_client: Some(minio_client),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app_state.clone())
            .service(upload_file)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
