use gb_core::Error as CoreError;
use redis::RedisError;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CoreError> for AuthError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError { .. } => AuthError::Internal(err.to_string()),
        }
    }
}
