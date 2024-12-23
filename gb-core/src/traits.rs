//! Core traits defining the system interfaces
//! File: gb-core/src/traits.rs

use crate::models::*;
use std::future::Future;
use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait InstanceStore {
    type Error;

    fn create(&self, instance: &Instance) -> impl Future<Output = Result<Instance, Self::Error>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Instance, Self::Error>> + Send;
    fn list_by_customer(&self, customer_id: Uuid) -> impl Future<Output = Result<Vec<Instance>, Self::Error>> + Send;
    fn update(&self, instance: &Instance) -> impl Future<Output = Result<Instance, Self::Error>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn list(&self, page: i32) -> impl Future<Output = Result<Vec<Instance>, Self::Error>> + Send;
}

#[async_trait]
pub trait RoomStore {
    type Error;

    fn create(&self, room: &Room) -> impl Future<Output = Result<Room, Self::Error>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Room, Self::Error>> + Send;
    fn list_by_instance(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<Room>, Self::Error>> + Send;
    fn update(&self, room: &Room) -> impl Future<Output = Result<Room, Self::Error>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn list(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<Room>, Self::Error>> + Send;
}

#[async_trait]
pub trait TrackStore {
    type Error;

    fn create(&self, track: &Track) -> impl Future<Output = Result<Track, Self::Error>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Track, Self::Error>> + Send;
    fn list_by_room(&self, room_id: Uuid) -> impl Future<Output = Result<Vec<Track>, Self::Error>> + Send;
    fn update(&self, track: &Track) -> impl Future<Output = Result<Track, Self::Error>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[async_trait]
pub trait UserStore {
    type Error;

    fn create(&self, user: &User) -> impl Future<Output = Result<User, Self::Error>> + Send;
    fn get(&self, id: Uuid) -> impl Future<Output = Result<User, Self::Error>> + Send;
    fn get_by_email(&self, email: &str) -> impl Future<Output = Result<User, Self::Error>> + Send;
    fn list_by_instance(&self, instance_id: Uuid) -> impl Future<Output = Result<Vec<User>, Self::Error>> + Send;
    fn update(&self, user: &User) -> impl Future<Output = Result<User, Self::Error>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[async_trait]
pub trait MessageStore {
    type Error;

    fn send_message(&self, message: &Message) -> impl Future<Output = Result<MessageId, Self::Error>> + Send;
    fn get_messages(&self, filter: &MessageFilter) -> impl Future<Output = Result<Vec<Message>, Self::Error>> + Send;
    fn update_status(&self, message_id: Uuid, status: Status) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn delete_messages(&self, filter: &MessageFilter) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn search_messages(&self, query: &SearchQuery) -> impl Future<Output = Result<Vec<Message>, Self::Error>> + Send;
}

#[async_trait]
pub trait FileStore {
    type Error;

    fn upload_file(&self, upload: &FileUpload) -> impl Future<Output = Result<FileInfo, Self::Error>> + Send;
    fn get_file(&self, file_id: Uuid) -> impl Future<Output = Result<FileContent, Self::Error>> + Send;
    fn delete_file(&self, file_id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn list_files(&self, prefix: &str) -> impl Future<Output = Result<Vec<FileInfo>, Self::Error>> + Send;
}
