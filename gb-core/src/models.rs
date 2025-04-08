use chrono::{DateTime, Utc};
use minio::s3::client::Client as MinioClient;
use rdkafka::producer::FutureProducer;
use redis::aio::ConnectionManager as RedisConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
//use zitadel::api::zitadel::auth::v1::auth_service_client::AuthServiceClient;
use serde_json::Value as JsonValue;
use std::str::FromStr;

use crate::config::AppConfig;

#[derive(Debug)]
pub struct CoreError(pub String);

// Add these near the top with other type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub status: String,
    pub shard_id: i32,
    pub region: String,
    pub config: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub config: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub instance_id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub kind: String,
    pub content: String,
    pub metadata: JsonValue,
    pub created_at: Option<DateTime<Utc>>,
    pub shard_key: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageFilter {
    pub conversation_id: Option<Uuid>,
    pub sender_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub conversation_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpload {
    pub content: Vec<u8>,
    pub filename: String,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub content: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub code: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl FromStr for UserStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            "suspended" => Ok(UserStatus::Suspended),
            _ => Ok(UserStatus::Inactive),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub media_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub instance_id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub status: UserStatus,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

// Update the Customer struct to include these fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub max_instances: u32,
    pub email: String,
    pub status: CustomerStatus,              // Add this field
    pub subscription_tier: SubscriptionTier, // Add this field
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Customer {
    pub fn new(
        name: String,
        email: String,
        subscription_tier: SubscriptionTier,
        max_instances: u32,
    ) -> Self {
        Customer {
            id: Uuid::new_v4(),
            name,
            email,
            max_instances,
            subscription_tier,
            status: CustomerStatus::Active, // Default to Active
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub instance_id: Uuid,
    pub name: String,
    pub max_participants: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub track_id: Uuid,
    pub subscriber_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStats {
    pub participant_count: u32,
    pub track_count: u32,
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

// App state shared across all handlers


// App state shared across all handlers
pub struct AppState {
    pub minio_client: Option<MinioClient>,
    pub config: Option<AppConfig>,
    pub db_pool: Option<PgPool>,
}

// File models
#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Conversation models
#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMember {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

// Calendar models
#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Task models
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

// Response models
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

// Error models
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Kafka error: {0}")]
    Kafka(String),

    #[error("Zitadel error: {0}")]
    Zitadel(#[from] tonic::Status),

    #[error("Minio error: {0}")]
    Minio(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let (status, error_message) = match self {
            AppError::Validation(_) => (actix_web::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized(_) => {
                (actix_web::http::StatusCode::UNAUTHORIZED, self.to_string())
            }
            AppError::Forbidden(_) => (actix_web::http::StatusCode::FORBIDDEN, self.to_string()),
            _ => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        actix_web::HttpResponse::build(status).json(ApiResponse::<()> {
            success: false,
            message: Some(error_message),
            data: None,
        })
    }
}
