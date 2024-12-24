use gb_core::Error as CoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<CoreError> for AuthError {
    fn from(err: CoreError) -> Self {
        AuthError::Internal(err.to_string())
    }
}