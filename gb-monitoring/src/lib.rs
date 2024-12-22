pub mod metrics;
pub mod logging;
pub mod telemetry;

pub use metrics::Metrics;
pub use logging::init_logging;
pub use telemetry::Telemetry;

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[tokio::test]
    async fn test_monitoring_integration() {
        // Initialize logging
        init_logging().unwrap();

        // Initialize metrics
        let metrics = Metrics::new().unwrap();

        // Initialize telemetry
        let telemetry = Telemetry::new("test-service").unwrap();

        // Test logging with metrics
        info!(
            active_connections = metrics.active_connections.get() as i64,
            "System initialized"
        );

        // Simulate some activity
        metrics.increment_connections();
        metrics.increment_messages();
        metrics.observe_request_duration(0.1);

        // Verify metrics
        assert_eq!(metrics.active_connections.get(), 1.0);
    }
}
