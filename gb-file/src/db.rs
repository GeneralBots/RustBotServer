use anyhow::Result;
use minio_rs::client::{Client as MinioClient, ClientBuilder as MinioClientBuilder};
use rdkafka::ClientConfig;
use rdkafka::producer::FutureProducer;
use redis::aio::ConnectionManager as RedisConnectionManager;
use sqlx::postgres::{PgPoolOptions, PgPool};
use zitadel::api::v1::auth::AuthServiceClient;

use crate::config::AppConfig;

pub async fn init_postgres(config: &AppConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
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

pub async fn init_zitadel(config: &AppConfig) -> Result<AuthServiceClient<tonic::transport::Channel>> {
    let channel = tonic::transport::Channel::from_shared(format!("https://{}", config.zitadel.domain))?
        .connect()
        .await?;
    
    let client = AuthServiceClient::new(channel);
    
    Ok(client)
}

pub async fn init_minio(config: &AppConfig) -> Result<MinioClient> {
    let client = MinioClientBuilder::new()
        .endpoint(&config.minio.endpoint)
        .access_key(&config.minio.access_key)
        .secret_key(&config.minio.secret_key)
        .ssl(config.minio.use_ssl)
        .build()?;
    
    // Ensure bucket exists
    if !client.bucket_exists(&config.minio.bucket).await? {
        client.make_bucket(&config.minio.bucket, None).await?;
    }
    
    Ok(client)
}
