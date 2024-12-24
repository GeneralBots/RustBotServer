use axum::{
    routing::{get, post},
    Router,
    extract::{
        ws::WebSocket,
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use gb_core::{Result, Error, models::*};
use gb_messaging::{MessageProcessor, models::MessageEnvelope};
use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use tracing::{instrument, error};
use uuid::Uuid;
use futures_util::StreamExt;

pub struct ApiState {
    pub message_processor: Mutex<MessageProcessor>,
}

pub fn create_router(message_processor: MessageProcessor) -> Router {
    let state = Arc::new(ApiState {
        message_processor: Mutex::new(message_processor),
    });

    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/messages", post(send_message))
        .route("/messages/:id", get(get_message))
        .route("/rooms", post(create_room))
        .route("/rooms/:id", get(get_room))
        .route("/rooms/:id/join", post(join_room))
        .route("/ws", get(websocket_handler))
        .with_state(state)
}

async fn handle_ws_connection(
    ws: WebSocket,
    state: Arc<ApiState>,
) -> Result<()> {
    let (_sender, mut receiver) = ws.split();
    
    while let Some(Ok(msg)) = receiver.next().await {
        if let Ok(text) = msg.to_text() {
            if let Ok(envelope) = serde_json::from_str::<MessageEnvelope>(text) {
                let mut processor = state.message_processor.lock().await;
                if let Err(e) = processor.process_messages().await {
                    error!("Failed to process message: {}", e);
                }
            }
        }
    }
    Ok(())
}

#[instrument(skip_all)]
async fn websocket_handler(
    State(state): State<Arc<ApiState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let _ = handle_ws_connection(socket, state).await;
    })
}

#[instrument(skip_all)]
async fn send_message(
    State(state): State<Arc<ApiState>>,
    Json(message): Json<Message>,
) -> Result<Json<MessageId>> {
    let envelope = MessageEnvelope {
        id: Uuid::new_v4(),
        message,
        metadata: HashMap::new(),
    };

    let mut processor = state.message_processor.lock().await;
    processor.process_messages().await
        .map_err(|e| Error::internal(format!("Failed to process message: {}", e)))?;

    Ok(Json(MessageId(envelope.id)))
}

#[instrument(skip_all)]
async fn get_message(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<Message>> {
    todo!()
}

#[instrument(skip_all)]
async fn create_room(
    State(_state): State<Arc<ApiState>>,
    Json(_config): Json<RoomConfig>,
) -> Result<Json<Room>> {
    todo!()
}

#[instrument(skip_all)]
async fn get_room(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<Room>> {
    todo!()
}

#[instrument(skip_all)]
async fn join_room(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
    Json(_user_id): Json<Uuid>,
) -> Result<Json<Connection>> {
    todo!()
}