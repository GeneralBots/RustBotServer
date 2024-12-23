use std::future::Future;
use uuid::Uuid;
use crate::errors::Result;
use crate::models::{
    Customer, Instance, Room, Track, User, Message, Connection,
    TrackInfo, Subscription, Participant, RoomStats, MessageId,
    MessageFilter, Status, SearchQuery, FileUpload, FileInfo,
    FileContent, RoomConfig
};

pub trait CustomerRepository: Send + Sync {
    fn create(&self, customer: &Customer) -> impl Future<Output = Result<Customer>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Customer>> + Send;
    fn update(&self, customer: &Customer) -> impl Future<Output = Result<Customer>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;
}

pub trait InstanceRepository: Send + Sync {
    fn create(&self, instance: &Instance) -> impl Future<Output = Result<Instance>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Instance>> + Send;
    fn get_by_customer(&self, customer_id: Uuid) -> impl Future<Output = Result<Vec<Instance>>> + Send;
    fn update(&self, instance: &Instance) -> impl Future<Output = Result<Instance>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;
    fn get_by_shard(&self, shard_id: i32) -> impl Future<Output = Result<Vec<Instance>>> + Send;
}

pub trait RoomRepository: Send + Sync {
    fn create(&self, room: &Room) -> impl Future<Output = Result<Room>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Room>> + Send;
    fn get_by_instance(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<Room>>> + Send;
    fn update(&self, room: &Room) -> impl Future<Output = Result<Room>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;
    fn get_active_rooms(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<Room>>> + Send;
}

pub trait TrackRepository: Send + Sync {
    fn create(&self, track: &Track) -> impl Future<Output = Result<Track>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Track>> + Send;
    fn get_by_room(&self, room_id: Uuid) -> impl Future<Output = Result<Vec<Track>>> + Send;
    fn update(&self, track: &Track) -> impl Future<Output = Result<Track>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;
}

pub trait UserRepository: Send + Sync {
    fn create(&self, user: &User) -> impl Future<Output = Result<User>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<User>> + Send;
    fn get_by_email(&self, email: &str) -> impl Future<Output = Result<User>> + Send;
    fn get_by_instance(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<User>>> + Send;
    fn update(&self, user: &User) -> impl Future<Output = Result<User>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;
}

pub trait RoomService: Send + Sync {
    fn create_room(&self, config: RoomConfig) -> impl Future<Output = Result<Room>> + Send;
    fn join_room(&self, room_id: Uuid, user_id: Uuid) -> impl Future<Output = Result<Connection>> + Send;
    fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> impl Future<Output = Result<()>> + Send;
    fn publish_track(&self, track: TrackInfo) -> impl Future<Output = Result<Track>> + Send;
    fn subscribe_track(&self, track_id: Uuid) -> impl Future<Output = Result<Subscription>> + Send;
    fn get_participants(&self, room_id: Uuid) -> impl Future<Output = Result<Vec<Participant>>> + Send;
    fn get_room_stats(&self, room_id: Uuid) -> impl Future<Output = Result<RoomStats>> + Send;
}

pub trait MessageService: Send + Sync {
    fn send_message(&self, message: Message) -> impl Future<Output = Result<MessageId>> + Send;
    fn get_messages(&self, filter: MessageFilter) -> impl Future<Output = Result<Vec<Message>>> + Send;
    fn update_status(&self, message_id: Uuid, status: Status) -> impl Future<Output = Result<()>> + Send;
    fn delete_messages(&self, filter: MessageFilter) -> impl Future<Output = Result<()>> + Send;
    fn search_messages(&self, query: SearchQuery) -> impl Future<Output = Result<Vec<Message>>> + Send;
}

pub trait FileService: Send + Sync {
    fn save_file(&self, file: FileUpload) -> impl Future<Output = Result<FileInfo>> + Send;
    fn get_file(&self, file_id: Uuid) -> impl Future<Output = Result<FileContent>> + Send;
    fn delete_file(&self, file_id: Uuid) -> impl Future<Output = Result<()>> + Send;
    fn list_files(&self, prefix: &str) -> impl Future<Output = Result<Vec<FileInfo>>> + Send;
}