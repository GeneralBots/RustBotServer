use minio::s3::Client;

use crate::services::config::AppConfig;


// App state shared across all handlers
pub struct AppState {
    pub minio_client: Option<Client>,
    pub config: Option<AppConfig>,
    pub db:  Option<sqlx::PgPool>,

}

