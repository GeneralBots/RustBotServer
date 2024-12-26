use gb_core::{Result, models::*};  // This will import both Message and MessageId

use gb_core::Error;
use uuid::Uuid;
use std::collections::HashMap;
use tracing::instrument;
use crate::MessageEnvelope;
use tokio::sync::broadcast;  // Add this import
use std::sync::Arc;
use tracing::{error, info};  // Add error and info macros here


pub struct MessageProcessor {
    tx: broadcast::Sender<MessageEnvelope>,
    rx: broadcast::Receiver<MessageEnvelope>,
    handlers: Arc<HashMap<String, Box<dyn Fn(MessageEnvelope) -> Result<()> + Send + Sync>>>,
}

impl Clone for MessageProcessor {
    fn clone(&self) -> Self {
        MessageProcessor {
            tx: self.tx.clone(), 
            rx: self.tx.subscribe(),
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl MessageProcessor {
    pub fn new() -> Self {
        Self::new_with_buffer_size(100)
    }

    pub fn new_with_buffer_size(buffer_size: usize) -> Self {
        let (tx, rx) = broadcast::channel(buffer_size);
        Self {
            tx,
            rx,
            handlers: Arc::new(HashMap::new()),
        }
    }

    pub fn sender(&self) -> broadcast::Sender<MessageEnvelope> {
        self.tx.clone()
    }

    #[instrument(skip(self, handler))]
    pub fn register_handler<F>(&mut self, kind: &str, handler: F) 
    where
        F: Fn(MessageEnvelope) -> Result<()> + Send + Sync + 'static,
    {
        Arc::get_mut(&mut self.handlers)
            .expect("Cannot modify handlers")
            .insert(kind.to_string(), Box::new(handler));
    }

    #[instrument(skip(self))]
    pub async fn add_message(&mut self, message: Message) -> Result<MessageId> {
        let envelope = MessageEnvelope {
            id: Uuid::new_v4(),
            message,
            metadata: HashMap::new(),
        };

        self.tx.send(envelope.clone())
            .map_err(|e| Error::internal(format!("Failed to queue message: {}", e)))?;

        // Start processing immediately
        if let Some(handler) = self.handlers.get(&envelope.message.kind) {
            handler(envelope.clone())
                .map_err(|e| Error::internal(format!("Handler error: {}", e)))?;
        }

        Ok(MessageId(envelope.id))
    }
    #[instrument(skip(self))]
    pub async fn process_messages(&mut self) -> Result<()> {
        while let Ok(envelope) = self.rx.recv().await {
            if let Some(handler) = self.handlers.get(&envelope.message.kind) {
                if let Err(e) = handler(envelope.clone()) {
                    error!("Handler error for message {}: {}", envelope.id, e);
                }
                    tracing::info!("Processing message: {:?}", &envelope.message.id);
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
    use gb_core::models::Message;
    use rstest::*;
    use uuid::Uuid;
    use std::{sync::Arc, time::Duration};
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
        let mut processor = MessageProcessor::new();
        let processed = Arc::new(Mutex::new(false));
        let processed_clone = processed.clone();

        processor.register_handler("test", move |envelope| {
            assert_eq!(envelope.message.content, "test content");
            let mut processed = processed_clone.blocking_lock();
            *processed = true;
            Ok(())
        });

        let mut processor_clone = processor.clone();
        let handle = tokio::spawn(async move {
            processor_clone.process_messages().await.unwrap();
        });

        let envelope = MessageEnvelope {
            id: Uuid::new_v4(),
            message: test_message,
            metadata: HashMap::new(),
        };

        processor.sender().send(envelope).unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
        
        assert!(*processed.lock().await);
        
        handle.abort();
    }
}
