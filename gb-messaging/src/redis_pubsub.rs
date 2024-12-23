use async_trait::async_trait;

use gb_core::{Result, Error};
use redis::{Client, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub struct RedisPubSub {
    client: Arc<Client>,
}

impl Clone for RedisPubSub {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl RedisPubSub {
    pub async fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| Error::redis(e.to_string()))?;

        // Test connection
        client.get_async_connection()
            .await
            .map_err(|e| Error::redis(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    #[instrument(skip(self, payload))]
    pub async fn publish<T>(&self, channel: &str, payload: &T) -> Result<()>
    where
        T: Serialize + std::fmt::Debug,
    {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| Error::redis(e.to_string()))?;

        let payload = serde_json::to_string(payload)
            .map_err(|e| Error::redis(e.to_string()))?;

        conn.publish(channel, payload)
            .await
            .map_err(|e| Error::redis(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use std::time::Duration;

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

        let pubsub_clone = redis_pubsub.clone();
        let test_message_clone = test_message.clone();
        
        let handle = tokio::spawn(async move {
            let handler = |msg: TestMessage| async move {
                assert_eq!(msg, test_message_clone);
                Ok(())
            };

            pubsub_clone.subscribe(&[channel], handler).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        redis_pubsub.publish(channel, &test_message)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
    }
}
