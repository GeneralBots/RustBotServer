use std::sync::Arc;

use minio::s3::Client;

use crate::services::{config::AppConfig, web_automation::BrowserPool};


#[derive(Clone)]
pub struct AppState {
    pub minio_client: Option<Client>,
    pub config: Option<AppConfig>,
    pub db:  Option<sqlx::PgPool>,
    pub db_custom:  Option<sqlx::PgPool>,
    pub browser_pool: Arc<BrowserPool>,
}

