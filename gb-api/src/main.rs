use gb_core::{Error, Result};
use tracing::{info, error};
use std::net::SocketAddr;
use gb_messaging::MessageProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    init_logging()?;
    
    // Initialize core components
    let app = initialize_bot_server().await?;
    
    // Start the server
    start_server(app).await
}

async fn initialize_bot_server() -> Result<axum::Router> {
    info!("Initializing General Bots server...");

    // Initialize the MessageProcessor
    let message_processor = MessageProcessor::new();
    
    // Build the Axum router using our router module
    let app = gb_api::create_router(message_processor)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    Ok(app)
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::EnvFilter;
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .init();

    Ok(())
}

async fn initialize_database() -> Result<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| Error::internal("DATABASE_URL not set".to_string()))?;

    sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| Error::internal(e.to_string()))
}

async fn initialize_redis() -> Result<redis::Client> {
    let redis_url = std::env::var("REDIS_URL")
        .map_err(|_| Error::internal("REDIS_URL not set".to_string()))?;

    redis::Client::open(redis_url)
        .map_err(|e| Error::internal(e.to_string()))
}

#[allow(dead_code)]
#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    redis: redis::Client,
}


async fn start_server(app: axum::Router) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("Starting server on {}", addr);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Listening on {}", addr);
            axum::serve(listener, app)
                .await
                .map_err(|e| Error::internal(format!("Server error: {}", e)))
        }
        Err(e) => {
            error!("Failed to bind to address: {}", e);
            Err(Error::internal(format!("Failed to bind to address: {}", e)))
       }
      
    }
}