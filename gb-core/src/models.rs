use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub subscription_tier: String,
    pub status: String,
    pub max_instances: i32,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub customer_id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub shard_key: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid, 
    pub kind: String,
    pub status: String,
    pub metadata: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub instance_id: Uuid,
    pub name: String,
    pub email: String,
    pub status: String,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

impl Customer {
    pub fn new(
        name: String,
        subscription_tier: String,
        max_instances: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            subscription_tier,
            status: "active".to_string(),
            max_instances,
            metadata: HashMap::new(),
            created_at: Utc::now()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_customer_creation() {
        let customer = Customer::new(
            "Test Corp".to_string(),
            "enterprise".to_string(),
            10,
        );

        assert_eq!(customer.name, "Test Corp");
        assert_eq!(customer.subscription_tier, "enterprise");
        assert_eq!(customer.max_instances, 10);
        assert_eq!(customer.status, "active");
    }
}
