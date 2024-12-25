use futures_util::SinkExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use gb_core::{Result, Error};

pub struct WebSocketClient {
    stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
    >,
}

impl WebSocketClient {
    fn to_gb_error(error: tokio_tungstenite::tungstenite::Error) -> Error {
        Error::new(
            gb_core::ErrorKind::WebSocket(error.to_string()),
            error.to_string()
        )
    }

    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url).await.map_err(Self::to_gb_error)?;
        Ok(Self {
            stream: ws_stream,
        })
    }

    pub async fn send_message(&mut self, payload: &str) -> Result<()> {
        self.stream
            .send(Message::Text(payload.to_string()))
            .await
            .map_err(Self::to_gb_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use rstest::*;
    use serde::{Deserialize, Serialize};
    use tokio_tungstenite::tungstenite::WebSocket;
    use std::time::Duration;
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
                let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
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
    async fn test_websocket(test_message: TestMessage) {
        let server_url = create_test_server().await;
        let mut client = WebSocketClient::connect(&server_url).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        client.send_message(&serde_json::to_string(&test_message).unwrap()).await.unwrap();
    }
}
