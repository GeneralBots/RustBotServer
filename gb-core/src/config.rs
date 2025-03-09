use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    // pub zitadel: ZitadelConfig,
    pub minio: MinioConfig,
    pub email: EmailConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZitadelConfig {
    pub domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
    pub bucket: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .expect("Invalid SERVER_PORT"),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .expect("Invalid DATABASE_MAX_CONNECTIONS"),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            },
            kafka: KafkaConfig {
                brokers: env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS must be set"),
            },
            // zitadel: ZitadelConfig {
            //     domain: env::var("ZITADEL_DOMAIN").expect("ZITADEL_DOMAIN must be set"),
            //     client_id: env::var("ZITADEL_CLIENT_ID").expect("ZITADEL_CLIENT_ID must be set"),
            //     client_secret: env::var("ZITADEL_CLIENT_SECRET")
            //         .expect("ZITADEL_CLIENT_SECRET must be set"),
            // },
            minio: MinioConfig {
                endpoint: env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set"),
                access_key: env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set"),
                secret_key: env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set"),
                use_ssl: env::var("MINIO_USE_SSL")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .expect("Invalid MINIO_USE_SSL"),
                bucket: env::var("MINIO_BUCKET").expect("MINIO_BUCKET must be set"),
            },
            email: EmailConfig {
                smtp_server: env::var("EMAIL_SMTP_SERVER").expect("EMAIL_SMTP_SERVER must be set"),
                smtp_port: env::var("EMAIL_SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .expect("Invalid EMAIL_SMTP_PORT"),
                username: env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME must be set"),
                password: env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set"),
                from_email: env::var("EMAIL_FROM").expect("EMAIL_FROM must be set"),
            },
        }
    }
}
