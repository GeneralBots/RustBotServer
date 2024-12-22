use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde::{Map, Value as JsonValue};
use uuid::Uuid;
use crate::{models::*, Result};

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, customer: &Customer) -> Result<Customer>;
    async fn get(&self, id: Uuid) -> Result<Customer>;
    async fn update(&self, customer: &Customer) -> Result<Customer>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait InstanceRepository: Send + Sync {
    async fn create(&self, instance: &Instance) -> Result<Instance>;
    async fn get(&self, id: Uuid) -> Result<Instance>;
    async fn get_by_customer(&self, customer_id: Uuid) -> Result<Vec<Instance>>;
    async fn update(&self, instance: &Instance) -> Result<Instance>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn get_by_shard(&self, shard_id: i32) -> Result<Vec<Instance>>;
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn create(&self, room: &Room) -> Result<Room>;
    async fn get(&self, id: Uuid) -> Result<Room>;
    async fn get_by_instance(&self, instance_id: Uuid) -> Result<Vec<Room>>;
    async fn update(&self, room: &Room) -> Result<Room>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn get_active_rooms(&self, instance_id: Uuid) -> Result<Vec<Room>>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> Result<Message>;
    async fn get(&self, id: Uuid) -> Result<Message>;
    async fn get_by_conversation(&self, conversation_id: Uuid) -> Result<Vec<Message>>;
    async fn update_status(&self, id: Uuid, status: String) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn get_by_shard(&self, shard_key: i32) -> Result<Vec<Message>>;
}

#[async_trait]
pub trait TrackRepository: Send + Sync {
    async fn create(&self, track: &Track) -> Result<Track>;
    async fn get(&self, id: Uuid) -> Result<Track>;
    async fn get_by_room(&self, room_id: Uuid) -> Result<Vec<Track>>;
    async fn update(&self, track: &Track) -> Result<Track>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User>;
    async fn get(&self, id: Uuid) -> Result<User>;
    async fn get_by_email(&self, email: &str) -> Result<User>;
    async fn get_by_instance(&self, instance_id: Uuid) -> Result<Vec<User>>;
    async fn update(&self, user: &User) -> Result<User>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait RoomService: Send + Sync {
    async fn create_room(&self, config: RoomConfig) -> Result<Room>;
    async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<Connection>;
    async fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()>;
    async fn publish_track(&self, track: TrackInfo) -> Result<Track>;
    async fn subscribe_track(&self, track_id: Uuid) -> Result<Subscription>;
    async fn get_participants(&self, room_id: Uuid) -> Result<Vec<Participant>>;
    async fn get_room_stats(&self, room_id: Uuid) -> Result<RoomStats>;
}

#[async_trait]
pub trait MessageService: Send + Sync {
    async fn send_message(&self, message: Message) -> Result<MessageId>;
    async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>>;
    async fn update_status(&self, message_id: Uuid, status: Status) -> Result<()>;
    async fn delete_messages(&self, filter: MessageFilter) -> Result<()>;
    async fn search_messages(&self, query: SearchQuery) -> Result<Vec<Message>>;
}

#[async_trait]
pub trait StorageService: Send + Sync {
    async fn save_file(&self, file: FileUpload) -> Result<FileInfo>;
    async fn get_file(&self, file_id: Uuid) -> Result<FileContent>;
    async fn delete_file(&self, file_id: Uuid) -> Result<()>;
    async fn list_files(&self, prefix: &str) -> Result<Vec<FileInfo>>;
}

#[async_trait]
pub trait MetricsService: Send + Sync {
    async fn record_metric(&self, metric: Metric) -> Result<()>;
    async fn get_metrics(&self, query: MetricsQuery) -> Result<Vec<MetricValue>>;
    async fn create_dashboard(&self, config: DashboardConfig) -> Result<Dashboard>;
    async fn get_dashboard(&self, id: Uuid) -> Result<Dashboard>;
}
