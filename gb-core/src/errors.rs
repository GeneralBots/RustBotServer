use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
        
#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Kafka error: {0}")]
    Kafka(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Messaging error: {0}")]
    Messaging(String),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn internal<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Internal(msg.to_string()), msg.to_string())
    }

    pub fn redis<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Redis(msg.to_string()), msg.to_string())
    }

    pub fn kafka<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Kafka(msg.to_string()), msg.to_string())
    }

    pub fn database<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Database(msg.to_string()), msg.to_string())
    }

    pub fn websocket<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::WebSocket(msg.to_string()), msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    MissingToken,
    InvalidCredentials,
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.kind {
            ErrorKind::NotFound(_) => StatusCode::NOT_FOUND,
            ErrorKind::Authentication(_) => StatusCode::UNAUTHORIZED,
            ErrorKind::Authorization(_) => StatusCode::FORBIDDEN,
            ErrorKind::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let body = Json(json!({
            "error": self.message,
            "kind": format!("{:?}", self.kind)
        }));

        (status, body).into_response()
    }
}
