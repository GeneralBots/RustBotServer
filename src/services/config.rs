use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub minio: MinioConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u32,
    pub database: String,
}

#[derive(Clone)]
pub struct MinioConfig {
    pub server: String,
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
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.server,
            self.database.port,
            self.database.database
        )
    }

    pub fn from_env() -> Self {
        let database = DatabaseConfig {
            username: env::var("TABLES_USERNAME").unwrap_or_else(|_| "user".to_string()),
            password: env::var("TABLES_PASSWORD").unwrap_or_else(|_| "pass".to_string()),
            server: env::var("TABLES_SERVER").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("TABLES_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            database: env::var("TABLES_DATABASE").unwrap_or_else(|_| "db".to_string()),
        };

        let minio = MinioConfig {
            server: env::var("DRIVE_SERVER").expect("DRIVE_SERVER not set"),
            access_key: env::var("DRIVE_ACCESSKEY").expect("DRIVE_ACCESSKEY not set"),
            secret_key: env::var("DRIVE_SECRET").expect("DRIVE_SECRET not set"),
            use_ssl: env::var("DRIVE_USE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            bucket: env::var("DRIVE_ORG_PREFIX").unwrap_or_else(|_| "".to_string()),
        };
        AppConfig {
            minio,
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
            database,
        }
    }
}
