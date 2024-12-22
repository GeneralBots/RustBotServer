use async_trait::async_trait;
use gb_core::{Result, Error, models::Message};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    consumer::{StreamConsumer, Consumer},
    ClientConfig, Message as KafkaMessage,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{instrument, error, info};
use uuid::Uuid;

pub struct KafkaBroker {
    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl KafkaBroker {
    pub fn new(brokers: &str, group_id: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| Error::Kafka(format!("Failed to create producer: {}", e)))?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| Error::Kafka(format!("Failed to create consumer: {}", e)))?;

        Ok(Self {
            producer,
            consumer,
        })
    }

    #[instrument(skip(self, value))]
    pub async fn publish<T: Serialize>(&self, topic: &str, key: &str, value: &T) -> Result<()> {
        let payload = serde_json::to_string(value)
            .map_err(|e| Error::Internal(format!("Serialization error: {}", e)))?;

        self.producer
            .send(
                FutureRecord::to(topic)
                    .key(key)
                    .payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| Error::Kafka(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self, handler))]
    pub async fn subscribe<T, F, Fut>(&self, topics: &[&str], handler: F) -> Result<()>
    where
        T: DeserializeOwned,
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        self.consumer
            .subscribe(topics)
            .map_err(|e| Error::Kafka(format!("Failed to subscribe: {}", e)))?;

        loop {
            match self.consumer.recv().await {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        match serde_json::from_slice::<T>(payload) {
                            Ok(value) => {
                                if let Err(e) = handler(value).await {
                                    error!("Handler error: {}", e);
                                }
                            }
                            Err(e) => error!("Deserialization error: {}", e),
                        }
                    }
                }
                Err(e) => error!("Consumer error: {}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    #[fixture]
    fn kafka_broker() -> KafkaBroker {
        KafkaBroker::new(
            "localhost:9092",
            "test-group",
        ).unwrap()
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
        kafka_broker: KafkaBroker,
        test_message: TestMessage,
    ) {
        let topic = "test-topic";
        let key = test_message.id.to_string();

        // Publish message
        kafka_broker.publish(topic, &key, &test_message)
            .await
            .unwrap();

        // Subscribe and verify
        let handler = |msg: TestMessage| async move {
            assert_eq!(msg, test_message);
            Ok(())
        };

        // Run subscription for a short time
        tokio::spawn(async move {
            kafka_broker.subscribe(&[topic], handler).await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
