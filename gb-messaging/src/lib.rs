mod kafka;
mod rabbitmq;
mod redis_pubsub;
mod websocket;
mod processor;
pub mod models;

pub use kafka::Kafka;
pub use rabbitmq::RabbitMQ;
pub use redis_pubsub::RedisPubSub;
pub use websocket::WebSocketClient;
pub use processor::MessageProcessor;
pub use models::MessageEnvelope;
mod broker;
pub use broker::KafkaBroker;

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::models::Message;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use std::sync::Arc;
    use redis::Client;
    use tokio::sync::broadcast;
    use std::collections::HashMap;
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    #[tokio::test]
    async fn test_messaging_integration() {
        let kafka = KafkaBroker::new(
            "localhost:9092",
            "test-group",
        );
                let redis_client = Client::open("redis://localhost")
                    .expect("Failed to create Redis client");
        let redis = RedisPubSub::new(Arc::new(redis_client));
        let rabbitmq = RabbitMQ::new("amqp://localhost:5672")
            .await
            .unwrap();

        let mut websocket = WebSocketClient::connect("ws://localhost:8080")
            .await
            .unwrap();

        let test_message = TestMessage {
            id: Uuid::new_v4(),
            content: "integration test".to_string(),
        };

        kafka.publish("test-topic", &test_message.id.to_string(), &test_message)
            .await
            .unwrap();

        redis.publish("test-channel", &test_message)
            .await
            .unwrap();

        rabbitmq.publish("", "test.key", &test_message)
            .await
            .unwrap();

        websocket.send_message(&serde_json::to_string(&test_message).unwrap())
            .await
            .unwrap();

        let mut processor = MessageProcessor::new();
        
        processor.register_handler("test", |envelope| {
            println!("Processed message: {}", envelope.message.content);
            Ok(())
        });

        let message = Message {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            kind: "test".to_string(),
            content: "test content".to_string(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: chrono::Utc::now(),
            shard_key: 0,
        };

        let envelope = MessageEnvelope {
            id: Uuid::new_v4(),
            message,
            metadata: std::collections::HashMap::new(),
        };

        processor.sender().send(envelope).unwrap();
    }
}
