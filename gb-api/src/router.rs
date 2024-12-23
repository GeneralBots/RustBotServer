use axum::{
    routing::{get, post},
    Router,
    extract::{
        ws::{WebSocket, Message as WsMessage},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use gb_core::{Result, Error, models::*};
use gb_messaging::{MessageProcessor, MessageEnvelope}; 
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, error};
use uuid::Uuid;

use futures_util::StreamExt;
use futures_util::SinkExt;

async fn handle_ws_connection(
    ws: WebSocket,
    State(_state): State<Arc<ApiState>>,
) -> Result<(), Error> {
    let (mut sender, mut receiver) = ws.split();
    // ... rest of the implementation
}


#[axum::debug_handler]
#[instrument(skip(state, ws))]
async fn websocket_handler(
    State(state): State<Arc<ApiState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let (mut sender, mut receiver) = socket.split();

        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(envelope) = serde_json::from_str::<MessageEnvelope>(text) {
                    let mut processor = state.message_processor.lock().await;
                    if let Err(e) = processor.sender().send(envelope).await {
                        error!("Failed to process WebSocket message: {}", e);
                    }
                }
            }
        }
    })
}

#[axum::debug_handler]
#[instrument(skip(state, message))]
async fn send_message(
    State(state): State<Arc<ApiState>>,
    Json(message): Json<Message>,
) -> Result<Json<MessageId>> {
    let envelope = MessageEnvelope {
        id: Uuid::new_v4(),
        message,
        metadata: std::collections::HashMap::new(),
    };

    let mut processor = state.message_processor.lock().await;
    processor.sender().send(envelope.clone()).await
        .map_err(|e| Error::internal(format!("Failed to send message: {}", e)))?;

    Ok(Json(MessageId(envelope.id)))
}

#[axum::debug_handler]
#[instrument(skip(state))]
async fn get_message(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Message>> {
    todo!()
}

#[axum::debug_handler]
#[instrument(skip(state, config))]
async fn create_room(
    State(state): State<Arc<ApiState>>,
    Json(config): Json<RoomConfig>,
) -> Result<Json<Room>> {
    todo!()
}

#[axum::debug_handler]
#[instrument(skip(state))]
async fn get_room(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Room>> {
    todo!()
}

#[axum::debug_handler]
#[instrument(skip(state))]
async fn join_room(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    Json(user_id): Json<Uuid>,
) -> Result<Json<Connection>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::body::Body;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_router(MessageProcessor::new(100));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_send_message() {
        let app = create_router(MessageProcessor::new(100));

        let message = Message {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            kind: "test".to_string(),
            content: "test message".to_string(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: chrono::Utc::now(),
            shard_key: 0,
        };

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/messages")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&message).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
