use gb_core::{Result, Error};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::ClientConfig;
use std::time::Duration;
use tracing::{instrument, error};
use serde::Serialize;
pub struct Kafka {
    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl Kafka {
    pub async fn new(brokers: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .map_err(|e| Error::kafka(format!("Failed to create producer: {}", e)))?;

        let consumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", "my-group")
            .create()
            .map_err(|e| Error::kafka(format!("Failed to create consumer: {}", e)))?;

        Ok(Self {
            producer,
            consumer,
        })
    }

    pub async fn publish<T: Serialize>(&self, topic: &str, message: &T) -> Result<()> {
        let payload = serde_json::to_string(message)
            .map_err(|e| Error::internal(format!("Serialization error: {}", e)))?;

        self.producer
            .send(
                FutureRecord::to(topic)
                    .payload(payload.as_bytes())
                    .key(""),
                Duration::from_secs(0),
            )
            .await
            .map_err(|(e, _)| Error::kafka(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    pub async fn subscribe(&self, topic: &str) -> Result<()> {
        self.consumer
            .subscribe(&[topic])
            .map_err(|e| Error::kafka(format!("Failed to subscribe: {}", e)))?;

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
    async fn kafka_broker() -> Kafka {
        Kafka::new("localhost:9092").await.unwrap()
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
    async fn test_publish_subscribe(#[future] kafka_broker: Kafka, test_message: TestMessage) {
        let topic = "test-topic";
        kafka_broker.publish(topic, &test_message)
            .await
            .unwrap();

        kafka_broker.subscribe(topic)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
