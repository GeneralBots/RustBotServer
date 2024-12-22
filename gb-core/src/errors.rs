use thiserror::Error;
use redis::RedisError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Kafka error: {0}")]
    Kafka(String),

    #[error("WebRTC error: {0}")]
    WebRTC(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Resource quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NotFound("User".to_string());
        assert_eq!(err.to_string(), "Not found: User");

        let err = Error::Unauthorized("Invalid token".to_string());
        assert_eq!(err.to_string(), "Unauthorized: Invalid token");

        let err = Error::QuotaExceeded("Max instances reached".to_string());
        assert_eq!(err.to_string(), "Resource quota exceeded: Max instances reached");
    }
}
