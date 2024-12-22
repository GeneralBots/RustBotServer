pub mod kafka;
pub mod redis_pubsub;
pub mod rabbitmq;
pub mod websocket;
pub mod processor;

pub use kafka::KafkaBroker;
pub use redis_pubsub::RedisPubSub;
pub use rabbitmq::RabbitMQ;
pub use websocket::WebSocketClient;
pub use processor::{MessageProcessor, MessageEnvelope};

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::models::Message;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    #[tokio::test]
    async fn test_messaging_integration() {
        // Initialize message brokers
        let kafka = KafkaBroker::new(
            "localhost:9092",
            "test-group",
        ).unwrap();

        let redis = RedisPubSub::new("redis://localhost")
            .await
            .unwrap();

        let rabbitmq = RabbitMQ::new("amqp://localhost:5672")
            .await
            .unwrap();

        let websocket = WebSocketClient::connect("ws://localhost:8080")
            .await
            .unwrap();

        // Create test message
        let test_message = TestMessage {
            id: Uuid::new_v4(),
            content: "integration test".to_string(),
        };

        // Test Kafka
        kafka.publish("test-topic", &test_message.id.to_string(), &test_message)
            .await
            .unwrap();

        // Test Redis PubSub
        redis.publish("test-channel", &test_message)
            .await
            .unwrap();

        // Test RabbitMQ
        rabbitmq.publish("", "test.key", &test_message)
            .await
            .unwrap();

        // Test WebSocket
        websocket.send(&test_message)
            .await
            .unwrap();

        // Test Message Processor
        let mut processor = MessageProcessor::new(100);
        
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

        processor.sender().send(envelope).await.unwrap();
    }
}
