use async_trait::async_trait;
use gb_core::{Result, Error};
use lapin::{
    options::*,
    types::FieldTable,
    Connection, ConnectionProperties,
    Channel, Consumer,
    message::Delivery,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, error};

pub struct RabbitMQ {
    connection: Connection,
    channel: Arc<Mutex<Channel>>,
}

impl RabbitMQ {
    pub async fn new(url: &str) -> Result<Self> {
        let connection = Connection::connect(
            url,
            ConnectionProperties::default(),
        )
        .await
        .map_err(|e| Error::Internal(format!("RabbitMQ connection error: {}", e)))?;

        let channel = connection.create_channel()
            .await
            .map_err(|e| Error::Internal(format!("RabbitMQ channel error: {}", e)))?;

        Ok(Self {
            connection,
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    #[instrument(skip(self, message))]
    pub async fn publish<T: Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
    ) -> Result<()> {
        let payload = serde_json::to_string(message)
            .map_err(|e| Error::Internal(format!("Serialization error: {}", e)))?;

        let channel = self.channel.lock().await;
        
        channel.basic_publish(
            exchange,
            routing_key,
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .map_err(|e| Error::Internal(format!("RabbitMQ publish error: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self, handler))]
    pub async fn subscribe<T, F, Fut>(
        &self,
        queue: &str,
        handler: F,
    ) -> Result<()>
    where
        T: DeserializeOwned,
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let channel = self.channel.lock().await;

        channel.queue_declare(
            queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| Error::Internal(format!("RabbitMQ queue declare error: {}", e)))?;

        let mut consumer = channel.basic_consume(
            queue,
            "consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| Error::Internal(format!("RabbitMQ consume error: {}", e)))?;

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Ok(payload) = String::from_utf8(delivery.data.clone()) {
                        match serde_json::from_str::<T>(&payload) {
                            Ok(value) => {
                                if let Err(e) = handler(value).await {
                                    error!("Handler error: {}", e);
                                }
                            }
                            Err(e) => error!("Deserialization error: {}", e),
                        }
                    }
                    delivery.ack(BasicAckOptions::default())
                        .await
                        .map_err(|e| error!("Ack error: {}", e)).ok();
                }
                Err(e) => error!("Consumer error: {}", e),
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
    async fn rabbitmq() -> RabbitMQ {
        RabbitMQ::new("amqp://localhost:5672")
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
        rabbitmq: RabbitMQ,
        test_message: TestMessage,
    ) {
        let queue = "test-queue";
        let routing_key = "test.key";

        // Subscribe first
        let rabbitmq_clone = rabbitmq.clone();
        let test_message_clone = test_message.clone();
        let handle = tokio::spawn(async move {
            let handler = |msg: TestMessage| async move {
                assert_eq!(msg, test_message_clone);
                Ok(())
            };

            rabbitmq_clone.subscribe(queue, handler).await.unwrap();
        });

        // Give subscription time to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Publish message
        rabbitmq.publish("", routing_key, &test_message)
            .await
            .unwrap();

        // Wait for handler to process
        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
    }
}
