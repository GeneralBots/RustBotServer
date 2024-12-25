use gb_core::{Error, Result};
use gb_core::models::Customer;
use tracing::{info, error};

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
    info!("Initializing General Bots...");

    // Initialize database connections
    let db_pool = initialize_database().await?;
    
    // Initialize Redis
    let redis_client = initialize_redis().await?;
    
    // Build the Axum router with our routes
    let app = axum::Router::new()
        .with_state(AppState {
            db: db_pool,
            redis: redis_client,
        })
        // Add your route handlers here
        .layer(tower_http::trace::TraceLayer::new_for_http());

    Ok(app)
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};
    
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
        .map_err(|_| Error::Configuration("DATABASE_URL not set".into()))?;

    sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| Error::Database(e.to_string()))
}

async fn initialize_redis() -> Result<redis::Client> {
    let redis_url = std::env::var("REDIS_URL")
        .map_err(|_| Error::Configuration("REDIS_URL not set".into()))?;

    redis::Client::open(redis_url)
        .map_err(|e| Error::Cache(e.to_string()))
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    redis: redis::Client,
}

async fn start_server(app: axum::Router) -> Result<()> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Error::Server(e.to_string()))?;

    Ok(())
}