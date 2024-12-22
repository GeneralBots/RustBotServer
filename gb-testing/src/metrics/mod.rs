use prometheus::{Registry, Counter, Gauge, Histogram, HistogramOpts};
use std::sync::Arc;

pub struct TestMetrics {
    registry: Registry,
    request_count: Counter,
    error_count: Counter,
    response_time: Histogram,
    active_users: Gauge,
}

impl TestMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let request_count = Counter::new("test_requests_total", "Total number of requests").unwrap();
        let error_count = Counter::new("test_errors_total", "Total number of errors").unwrap();
        let response_time = Histogram::with_opts(
            HistogramOpts::new("test_response_time", "Response time in seconds")
        ).unwrap();
        let active_users = Gauge::new("test_active_users", "Number of active users").unwrap();

        registry.register(Box::new(request_count.clone())).unwrap();
        registry.register(Box::new(error_count.clone())).unwrap();
        registry.register(Box::new(response_time.clone())).unwrap();
        registry.register(Box::new(active_users.clone())).unwrap();

        Self {
            registry,
            request_count,
            error_count,
            response_time,
            active_users,
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.inc();
    }

    pub fn increment_errors(&self) {
        self.error_count.inc();
    }

    pub fn observe_response_time(&self, duration: f64) {
        self.response_time.observe(duration);
    }

    pub fn set_active_users(&self, count: i64) {
        self.active_users.set(count as f64);
    }
}
