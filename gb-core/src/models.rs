//! Core domain models for the general-bots system
//! File: gb-core/src/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub struct CoreError(pub String);

// Add these near the top with other type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise
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
            _ => Ok(UserStatus::Inactive)
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
    pub status: CustomerStatus,          // Add this field
    pub subscription_tier: SubscriptionTier,  // Add this field
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
            status: CustomerStatus::Active,  // Default to Active
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
