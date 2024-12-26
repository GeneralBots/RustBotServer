mod logging;
mod metrics;
mod telemetry;

pub use logging::init_logging;
pub use metrics::Metrics;
pub use telemetry::Telemetry;


#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[tokio::test]
    async fn test_monitoring_integration() {
        // Initialize logging
        init_logging("gb").unwrap();

        // Initialize metrics
        let metrics = Metrics::new();

        // Initialize telemetry
        Telemetry::new("test-service").await.unwrap();

        // Test logging with metrics
        info!(
            active_connections = metrics.active_connections.get() as i64,
            "System initialized"
        );

        // Simulate some activity
        metrics.set_active_connections(1);
        metrics.increment_message_count();
        metrics.observe_processing_time(0.1);

        // Verify metrics
        assert_eq!(metrics.active_connections.get(), 1);
    }
}
