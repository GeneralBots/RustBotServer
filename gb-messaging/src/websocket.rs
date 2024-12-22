use gb_core::{Result, Error};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::Mutex,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    WebSocketStream,
};
use tracing::{instrument, error};

pub struct WebSocketClient {
    write: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    read: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| Error::Internal(format!("WebSocket connection error: {}", e)))?;

        let (write, read) = ws_stream.split();

        Ok(Self {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
        })
    }

    #[instrument(skip(self, message))]
    pub async fn send<T: Serialize>(&self, message: &T) -> Result<()> {
        let payload = serde_json::to_string(message)
            .map_err(|e| Error::Internal(format!("Serialization error: {}", e)))?;

        let mut write = self.write.lock().await;
        write.send(Message::Text(payload))
            .await
            .map_err(|e| Error::Internal(format!("WebSocket send error: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self, handler))]
    pub async fn receive<T, F, Fut>(&self, handler: F) -> Result<()>
    where
        T: DeserializeOwned,
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut read = self.read.lock().await;

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(payload)) => {
                    match serde_json::from_str::<T>(&payload) {
                        Ok(value) => {
                            if let Err(e) = handler(value).await {
                                error!("Handler error: {}", e);
                            }
                        }
                        Err(e) => error!("Deserialization error: {}", e),
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => error!("WebSocket receive error: {}", e),
                _ => continue,
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
    use tokio::net::TcpListener;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: Uuid,
        content: String,
    }

    async fn create_test_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let ws_stream = tokio_tungstenite::accept_async(stream)
                    .await
                    .unwrap();
                
                let (mut write, mut read) = ws_stream.split();
                
                while let Some(Ok(msg)) = read.next().await {
                    if let Message::Text(_) = msg {
                        write.send(msg).await.unwrap();
                    }
                }
            }
        });

        format!("ws://{}", addr)
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
    async fn test_websocket_client(test_message: TestMessage) {
        let server_url = create_test_server().await;
        let client = WebSocketClient::connect(&server_url).await.unwrap();
        let test_message_clone = test_message.clone();

        // Start receiving messages
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let handler = |msg: TestMessage| async move {
                assert_eq!(msg, test_message_clone);
                Ok(())
            };

            client_clone.receive(handler).await.unwrap();
        });

        // Give receiver time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send test message
        client.send(&test_message).await.unwrap();

        // Wait for message to be processed
        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
    }
}
