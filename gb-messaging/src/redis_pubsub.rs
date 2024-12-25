use gb_core::{Result, Error};
use redis::{Client, AsyncCommands, aio::PubSub};
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;
use futures_util::StreamExt;

#[derive(Clone)]
pub struct RedisPubSub {
    client: Arc<Client>,
}

impl RedisPubSub {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    #[instrument(skip(self, payload), err)]
    pub async fn publish<T: Serialize>(&self, channel: &str, payload: &T) -> Result<()> {
        let mut conn = self.client
            .get_async_connection()
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        let serialized = serde_json::to_string(payload)
            .map_err(|e| Error::internal(e.to_string()))?;

        conn.publish::<_, _, i32>(channel, serialized)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self, handler), err)]
    pub async fn subscribe<F>(&self, channels: &[&str], mut handler: F) -> Result<()>
    where
        F: FnMut(String, String) + Send + 'static,
    {
        let mut pubsub = self.client
            .get_async_connection()
            .await
            .map_err(|e| Error::internal(e.to_string()))?
            .into_pubsub();

        for channel in channels {
            pubsub.subscribe(*channel)
                .await
                .map_err(|e| Error::internal(e.to_string()))?;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload()
                .map_err(|e| Error::internal(e.to_string()))?;

            handler(channel, payload);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::Client;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use tokio::sync::mpsc;
    use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    async fn setup() -> (RedisPubSub, TestMessage) {
        let client = Arc::new(Client::open("redis://127.0.0.1/").unwrap());
        let redis_pubsub = RedisPubSub::new(client);
        
        let test_message = TestMessage {
            id: Uuid::new_v4(),
            content: "test message".to_string(),
        };

        (redis_pubsub, test_message)
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let (redis_pubsub, test_message) = setup().await;
        let channel = "test_channel";
        let (tx, mut rx) = mpsc::channel(1);

        let pubsub_clone = redis_pubsub.clone();
        let test_message_clone = test_message.clone();

        tokio::spawn(async move {
            let handler = move |_channel: String, payload: String| {
                let received: TestMessage = serde_json::from_str(&payload).unwrap();
                tx.try_send(received).unwrap();
            };

            pubsub_clone.subscribe(&[channel], handler).await.unwrap();
        });

        // Give the subscriber time to connect
        tokio::time::sleep(Duration::from_millis(100)).await;

        redis_pubsub.publish(channel, &test_message).await.unwrap();

        let received = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(received, test_message);
    }
}
