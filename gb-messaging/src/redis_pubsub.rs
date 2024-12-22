use async_trait::async_trait;
use gb_core::{Result, Error};
use redis::{
    aio::MultiplexedConnection,
    AsyncCommands, Client,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, error};

pub struct RedisPubSub {
    client: Client,
    conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisPubSub {
    pub async fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| Error::Redis(e))?;

        let conn = client.get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Redis(e))?;

        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    #[instrument(skip(self, message))]
    pub async fn publish<T: Serialize>(&self, channel: &str, message: &T) -> Result<()> {
        let payload = serde_json::to_string(message)
            .map_err(|e| Error::Internal(format!("Serialization error: {}", e)))?;

        let mut conn = self.conn.lock().await;
        conn.publish(channel, payload)
            .await
            .map_err(|e| Error::Redis(e))?;

        Ok(())
    }

    #[instrument(skip(self, handler))]
    pub async fn subscribe<T, F, Fut>(&self, channels: &[&str], handler: F) -> Result<()>
    where
        T: DeserializeOwned,
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut pubsub = self.client.get_async_connection()
            .await
            .map_err(|e| Error::Redis(e))?
            .into_pubsub();

        for channel in channels {
            pubsub.subscribe(*channel)
                .await
                .map_err(|e| Error::Redis(e))?;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload()
                .map_err(|e| Error::Redis(e))?;

            match serde_json::from_str::<T>(&payload) {
                Ok(value) => {
                    if let Err(e) = handler(value).await {
                        error!("Handler error: {}", e);
                    }
                }
                Err(e) => error!("Deserialization error: {}", e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    #[fixture]
    async fn redis_pubsub() -> RedisPubSub {
        RedisPubSub::new("redis://localhost")
            .await
            .unwrap()
    }

    #[fixture]
    fn test_message() -> TestMessage {
        TestMessage {
            id: Uuid::new_v4(),
            content: "test message".to_string(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_publish_subscribe(
        redis_pubsub: RedisPubSub,
        test_message: TestMessage,
    ) {
        let channel = "test-channel";

        // Subscribe first
        let pubsub_clone = redis_pubsub.clone();
        let test_message_clone = test_message.clone();
        
        let handle = tokio::spawn(async move {
            let handler = |msg: TestMessage| async move {
                assert_eq!(msg, test_message_clone);
                Ok(())
            };

            pubsub_clone.subscribe(&[channel], handler).await.unwrap();
        });

        // Give subscription time to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Publish message
        redis_pubsub.publish(channel, &test_message)
            .await
            .unwrap();

        // Wait for handler to process
        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
    }
}
