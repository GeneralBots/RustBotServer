pub mod router;

pub use router::{create_router, ApiState};

#[cfg(test)]
mod tests {
    use super::*;
    use gb_messaging::MessageProcessor;
    use axum::Router;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_api_integration() {
        // Initialize message processor
        let processor = MessageProcessor::new();
        
        // Create router
        let app: Router = create_router(processor);

        // Test health endpoint
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test message sending
        let message = gb_core::models::Message {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            kind: "test".to_string(),
            content: "integration test".to_string(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: Some(chrono::Utc::now()),
            shard_key: Some(0),
        };

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/messages")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_string(&message).unwrap()
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
