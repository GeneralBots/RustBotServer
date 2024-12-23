use prometheus::{IntCounter, IntGauge, Histogram, Registry};

pub struct Metrics {
    registry: Registry,
    message_counter: IntCounter,
    active_connections: IntGauge,
    message_processing_time: Histogram,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let message_counter = IntCounter::new(
            "message_total",
            "Total number of messages processed"
        ).unwrap();
        
        let active_connections = IntGauge::new(
            "active_connections",
            "Number of active connections"
        ).unwrap();
        
        let message_processing_time = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "message_processing_seconds",
                "Time spent processing messages"
            ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0])
        ).unwrap();

        registry.register(Box::new(message_counter.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(message_processing_time.clone())).unwrap();

        Self {
            registry,
            message_counter,
            active_connections,
            message_processing_time,
        }
    }

    pub fn increment_message_count(&self) {
        self.message_counter.inc();
    }

    pub fn observe_processing_time(&self, duration_seconds: f64) {
        self.message_processing_time.observe(duration_seconds);
    }

    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics() {
        let metrics = Metrics::new();
        
        metrics.increment_message_count();
        assert_eq!(metrics.message_counter.get(), 1);
        
        metrics.set_active_connections(10);
        assert_eq!(metrics.active_connections.get(), 10);
        
        metrics.observe_processing_time(0.5);
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&metrics.registry().gather(), &mut buffer).unwrap();
        assert!(!buffer.is_empty());
    }
}
