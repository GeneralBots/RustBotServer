use crate::config::AppConfig;
use anyhow::Result;
use minio::s3::client::{Client as MinioClient, ClientBuilder as MinioClientBuilder};
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use redis::aio::ConnectionManager as RedisConnectionManager;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::str::FromStr;
// use zitadel::api::clients::ClientBuilder;
// use zitadel::api::interceptors::AccessTokenInterceptor;
// use zitadel::api::zitadel::auth::v1::auth_service_client::AuthServiceClient;

pub async fn init_postgres(config: &AppConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(pool)
}

pub async fn init_redis(config: &AppConfig) -> Result<RedisConnectionManager> {
    let client = redis::Client::open(config.redis.url.as_str())?;
    let connection_manager = RedisConnectionManager::new(client).await?;

    Ok(connection_manager)
}

pub async fn init_kafka(config: &AppConfig) -> Result<FutureProducer> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka.brokers)
        .set("message.timeout.ms", "5000")
        .create()?;

    Ok(producer)
}

pub async fn init_zitadel(
    _config: &AppConfig,
) -> Result<
    (),
    Box<dyn std::error::Error>>
 {
    
    // TODO: https://github.com/smartive/zitadel-rust/blob/be389ca08c7f82d36fc1bcc36d2d9eb8666b22cd/examples/fetch_profile_with_service_account.rs#L18
    Ok(())
}

pub async fn init_minio(
    config: &AppConfig,
) -> Result<MinioClient, Box<dyn std::error::Error + Send + Sync>> {
    // Construct the base URL
    let base_url = format!("https://{}", config.minio.endpoint);
    let base_url = BaseUrl::from_str(&base_url)?;

    // Create credentials provider
    let credentials = StaticProvider::new(&config.minio.access_key, &config.minio.secret_key, None);

    // Build the MinIO client
    let client = MinioClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(credentials)))
        //.secure(config.minio.use_ssl)
        .build()?;

    Ok(client)
}
