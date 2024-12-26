use gb_core::Error;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::config::ClientConfig;
use std::time::Duration;
use serde::Serialize;

#[allow(dead_code)]
pub struct KafkaBroker {
    producer: FutureProducer,
    // Stored for reconnection logic
    broker_address: String,
    // Stored for consumer group management
    group_id: String,
}

impl KafkaBroker {
    pub fn new(broker_address: &str, group_id: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker_address)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation failed");

        Self {
            producer,
            broker_address: broker_address.to_string(),
            group_id: group_id.to_string(),
        }
    }

    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        key: &str,
        message: &T,
    ) -> Result<(), Error> {
        let payload = serde_json::to_string(message)
            .map_err(|e| Error::internal(format!("Serialization failed: {}", e)))?;

        self.producer
            .send(
                FutureRecord::to(topic)
                    .key(key)
                    .payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| Error::internal(format!("Failed to publish message: {}", e)))?;

        Ok(())
    }

}