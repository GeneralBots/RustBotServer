use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub minio: MinioConfig,
    pub server: ServerConfig,
}

#[derive(Clone)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
    pub bucket: String,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let minio = MinioConfig {
            endpoint: env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT not set"),
            access_key: env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY not set"),
            secret_key: env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY not set"),
            use_ssl: env::var("MINIO_USE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            bucket: env::var("MINIO_BUCKET").expect("MINIO_BUCKET not set"),
        };
        AppConfig { 
            minio,
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT").ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
        }
    }
}
