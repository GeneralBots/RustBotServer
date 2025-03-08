use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use gb_core::{models::*, Error, Result};
use gb_messaging::{models::MessageEnvelope, MessageProcessor};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, instrument};
use uuid::Uuid;

pub fn create_router(message_processor: AppState) -> Router {
    let state = Arc::new(ApiState {
        message_processor: Mutex::new(message_processor),
    });
    Router::new()
        .route("/monitoring/metrics", get(get_monitoring_metrics))
        .with_state(state)
}
