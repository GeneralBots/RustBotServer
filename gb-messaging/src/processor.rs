use gb_core::{Result, Error, models::Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{instrument, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: Uuid,
    pub message: Message,
    pub metadata: HashMap<String, String>,
}

pub struct MessageProcessor {
    tx: mpsc::Sender<MessageEnvelope>,
    rx: mpsc::Receiver<MessageEnvelope>,
    handlers: HashMap<String, Box<dyn Fn(MessageEnvelope) -> Result<()> + Send + Sync>>,
}

impl MessageProcessor {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        
        Self {
            tx,
            rx,
            handlers: HashMap::new(),
        }
    }

    pub fn sender(&self) -> mpsc::Sender<MessageEnvelope> {
        self.tx.clone()
    }

    #[instrument(skip(self, handler))]
    pub fn register_handler<F>(&mut self, kind: &str, handler: F)
    where
        F: Fn(MessageEnvelope) -> Result<()> + Send + Sync + 'static,
    {
        self.handlers.insert(kind.to_string(), Box::new(handler));
    }

    #[instrument(skip(self))]
    pub async fn process_messages(&mut self) -> Result<()> {
        while let Some(envelope) = self.rx.recv().await {
            if let Some(handler) = self.handlers.get(&envelope.message.kind) {
                if let Err(e) = handler(envelope.clone()) {
                    error!("Handler error for message {}: {}", envelope.id, e);
                }
            } else {
                error!("No handler registered for message kind: {}", envelope.message.kind);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[fixture]
    fn test_message() -> Message {
        Message {
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
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_message_processor(test_message: Message) {
        let mut processor = MessageProcessor::new(100);
        let processed = Arc::new(Mutex::new(false));
        let processed_clone = processed.clone();

        // Register handler
        processor.register_handler("test", move |envelope| {
            assert_eq!(envelope.message.content, "test content");
            let mut processed = processed_clone.blocking_lock();
            *processed = true;
            Ok(())
        });

        // Start processing in background
        let mut processor_clone = processor.clone();
        let handle = tokio::spawn(async move {
            processor_clone.process_messages().await.unwrap();
        });

        // Send test message
        let envelope = MessageEnvelope {
            id: Uuid::new_v4(),
            message: test_message,
            metadata: HashMap::new(),
        };

        processor.sender().send(envelope).await.unwrap();

        // Wait for processing
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Verify message was processed
        assert!(*processed.lock().await);
        
        handle.abort();
    }
}
