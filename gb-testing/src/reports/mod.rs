use anyhow;
use serde::Serialize;
use std::time::{Duration, SystemTime};

#[derive(Debug, Serialize)]
pub struct TestReport {
    pub timestamp: SystemTime,
    pub duration: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: f64,
    pub percentiles: Percentiles,
    pub errors: Vec<TestError>,
}

#[derive(Debug, Serialize)]
pub struct Percentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Serialize)]
pub struct TestError {
    pub error_type: String,
    pub message: String,
    pub count: u64,
}

impl TestReport {
    pub fn new(
        duration: Duration,
        total_requests: u64,
        successful_requests: u64,
        failed_requests: u64,
        avg_response_time: f64,
        percentiles: Percentiles,
        errors: Vec<TestError>,
    ) -> Self {
        Self {
            timestamp: SystemTime::now(),
            duration,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time,
            percentiles,
            errors,
        }
    }

    pub fn save_json(&self, path: &str) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn save_html(&self, path: &str) -> anyhow::Result<()> {
        // HTML report generation implementation
        Ok(())
    }
}
