use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[async_trait]
pub trait TestScenario {
    async fn setup(&mut self) -> anyhow::Result<()>;
    async fn execute(&self) -> anyhow::Result<()>;
    async fn teardown(&mut self) -> anyhow::Result<()>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub name: String,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub metrics: ScenarioMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioMetrics {
    pub requests: u64,
    pub failures: u64,
    pub avg_response_time: f64,
    pub max_response_time: f64,
    pub min_response_time: f64,
    pub throughput: f64,
}
