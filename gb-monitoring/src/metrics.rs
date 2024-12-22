use gb_core::{Result, Error};
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, Registry,
    opts, register_counter, register_gauge, register_histogram,
};
use std::sync::Arc;
use tracing::{instrument, error};

#[derive(Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
    active_connections: Gauge,
    message_count: IntCounter,
    request_duration: Histogram,
    active_rooms: Gauge,
    media_bandwidth: Gauge,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let active_connections = register_gauge!(
            opts!("gb_active_connections", "Number of active connections"),
            registry
        ).map_err(|e| Error::Internal(format!("Failed to create metric: {}", e)))?;

        let message_count = register_counter!(
            opts!("gb_message_count", "Total number of messages processed"),
            registry
        ).map_err(|e| Error::Internal(format!("Failed to create metric: {}", e)))?;

        let request_duration = register_histogram!(
            HistogramOpts::new(
                "gb_request_duration",
                "Request duration in seconds"
            ),
            registry
        ).map_err(|e| Error::Internal(format!("Failed to create metric: {}", e)))?;

        let active_rooms = register_gauge!(
            opts!("gb_active_rooms", "Number of active rooms"),
            registry
        ).map_err(|e| Error::Internal(format!("Failed to create metric: {}", e)))?;

        let media_bandwidth = register_gauge!(
            opts!("gb_media_bandwidth", "Current media bandwidth usage in bytes/sec"),
            registry
        ).map_err(|e| Error::Internal(format!("Failed to create metric: {}", e)))?;

        Ok(Self {
            registry: Arc::new(registry),
            active_connections,
            message_count,
            request_duration,
            active_rooms,
            media_bandwidth,
        })
    }

    #[instrument(skip(self))]
    pub fn increment_connections(&self) {
        self.active_connections.inc();
    }

    #[instrument(skip(self))]
    pub fn decrement_connections(&self) {
        self.active_connections.dec();
    }

    #[instrument(skip(self))]
    pub fn increment_messages(&self) {
        self.message_count.inc();
    }

    #[instrument(skip(self))]
    pub fn observe_request_duration(&self, duration: f64) {
        self.request_duration.observe(duration);
    }

    #[instrument(skip(self))]
    pub fn set_active_rooms(&self, count: i64) {
        self.active_rooms.set(count as f64);
    }

    #[instrument(skip(self))]
    pub fn set_media_bandwidth(&self, bytes_per_sec: f64) {
        self.media_bandwidth.set(bytes_per_sec);
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::core::{Collector, Desc};

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();
        
        // Test increment connections
        metrics.increment_connections();
        assert_eq!(
            metrics.active_connections.get(),
            1.0
        );

        // Test decrement connections
        metrics.decrement_connections();
        assert_eq!(
            metrics.active_connections.get(),
            0.0
        );

        // Test message count
        metrics.increment_messages();
        assert_eq!(
            metrics.message_count.get(),
            1
        );

        // Test request duration
        metrics.observe_request_duration(0.5);
        let mut buffer = Vec::new();
        metrics.request_duration.encode(&mut buffer).unwrap();
        assert!(!buffer.is_empty());

        // Test active rooms
        metrics.set_active_rooms(10);
        assert_eq!(
            metrics.active_rooms.get(),
            10.0
        );

        // Test media bandwidth
        metrics.set_media_bandwidth(1024.0);
        assert_eq!(
            metrics.media_bandwidth.get(),
            1024.0
        );
    }
}
