use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "guest" => UserRole::Guest,
            _ => UserRole::User,
        }
    }
}

impl From<String> for UserStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "inactive" => UserStatus::Inactive,
            "suspended" => UserStatus::Suspended,
            _ => UserStatus::Active,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}