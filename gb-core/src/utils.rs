use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::models::{ApiResponse, AppError, User};

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // subject (user ID)
    pub exp: usize,         // expiration time
    pub iat: usize,         // issued at
    pub email: String,      // user email
    pub username: String,   // username
}

// Generate JWT token
pub fn generate_jwt(user: &User, secret: &str) -> Result<String, AppError> {
    let expiration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 86400; // 24 hours
    
    let issued_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
        iat: issued_at,
        email: user.email.clone(),
        username: user.email.clone(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate JWT: {}", e)))
}

// Validate JWT token
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::default();
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
}

// Extract user ID from request
pub fn extract_user_id(req: &HttpRequest) -> Result<Uuid, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;
    
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid Authorization header format".to_string()));
    }
    
    let token = &auth_header[7..];
    let claims = validate_jwt(token, "your-secret-key")?;
    
    Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))
}

// Send email
pub async fn send_email(
    config: &AppConfig,
    to_email: &str,
    subject: &str,
    body: &str,
) -> Result<(), AppError> {
    let email = Message::builder()
        .from(config.email.from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to create email: {}", e)))?;

    let creds = Credentials::new(
        config.email.username.clone(),
        config.email.password.clone(),
    );

    // Open a remote connection to the SMTP server
    let mailer = SmtpTransport::relay(&config.email.smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
        .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;

    Ok(())
}

// Send message to Kafka
pub async fn send_to_kafka(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    payload: &str,
) -> Result<(), AppError> {
    producer
        .send(
            FutureRecord::to(topic)
                .key(key)
                .payload(payload),
            Timeout::After(Duration::from_secs(5)),
        )
        .await
        .map_err(|(e, _)| AppError::Kafka(format!("Failed to send message to Kafka: {}", e)))?;
    
    Ok(())
}

// Format datetime for JSON responses
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

// Create a standard API response
pub fn create_response<T: Serialize>(
    data: T,
    message: Option<String>,
) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message,
        data: Some(data),
    })
}
