use gb_core::{Result, Error};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::ClientConfig;
use std::time::Duration;
use serde::Serialize;

#[allow(dead_code)]
pub struct Kafka {
    broker_address: String,
    group_id: String,

    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl Kafka {
    pub async fn new(broker_address: &str, group_id: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", broker_address)
            .create()
            .map_err(|e| Error::kafka(format!("Failed to create producer: {}", e)))?;

        
        let consumer = ClientConfig::new() 
            .set("bootstrap.servers", broker_address)
            .set("group.id", group_id)
            .create()
            .map_err(|e| Error::kafka(format!("Failed to create consumer: {}", e)))?;

        Ok(Self {
            broker_address: broker_address.to_string(),
            group_id: group_id.to_string(),
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
    use tokio;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    #[fixture]
    fn test_message() -> TestMessage {
        TestMessage {
            id: Uuid::new_v4(),
            content: "test message".to_string(),
        }
    }
    
    #[fixture]
    async fn kafka() -> Kafka {
        Kafka::new(
            "localhost:9092",
            "test-group",
        ).await.unwrap()
    }
    
    #[rstest]
    #[tokio::test]
    async fn test_publish_subscribe(
        #[future] kafka: Kafka,
        test_message: TestMessage
    ) {
        let topic = "test-topic";
        let kafka = kafka.await;
        kafka.publish(topic, &test_message)
            .await
            .unwrap();

        kafka.subscribe(topic)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
