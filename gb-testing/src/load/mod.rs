use goose::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTestConfig {
    pub users: usize,
    pub duration: std::time::Duration,
    pub ramp_up: std::time::Duration,
    pub scenarios: Vec<String>,
}

pub struct LoadTest {
    pub config: LoadTestConfig,
    pub metrics: crate::metrics::TestMetrics,
}

impl LoadTest {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            metrics: crate::metrics::TestMetrics::new(),
        }
    }

    pub async fn run(&self) -> anyhow::Result<crate::reports::TestReport> {
        let mut goose = GooseAttack::initialize()?;

        goose
            .set_default_host("http://localhost:8080")?
            .set_users(self.config.users)?
            .set_startup_time(self.config.ramp_up)?
            .set_run_time(self.config.duration)?;

        for scenario in &self.config.scenarios {
            match scenario.as_str() {
                "auth" => goose.register_scenario(auth_scenario()),
                "api" => goose.register_scenario(api_scenario()),
                "webrtc" => goose.register_scenario(webrtc_scenario()),
                _ => continue,
            }?;
        }

        let metrics = goose.execute().await?;
        Ok(crate::reports::TestReport::from(metrics))
    }
}

fn auth_scenario() -> Scenario {
    scenario!("Authentication")
        .register_transaction(transaction!(login))
        .register_transaction(transaction!(logout))
}

async fn login(user: &mut GooseUser) -> TransactionResult {
    let payload = serde_json::json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let _response = user
        .post_json("/auth/login", &payload)
        .await?
        .response?;

    Ok(())
}

async fn logout(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .post("/auth/logout")
        .await?
        .response?;

    Ok(())
}

fn api_scenario() -> Scenario {
    scenario!("API")
        .register_transaction(transaction!(create_instance))
        .register_transaction(transaction!(list_instances))
}

async fn create_instance(user: &mut GooseUser) -> TransactionResult {
    let payload = serde_json::json!({
        "name": "test-instance",
        "config": {
            "memory": "512Mi",
            "cpu": "0.5"
        }
    });

    let _response = user
        .post_json("/api/instances", &payload)
        .await?
        .response?;

    Ok(())
}

async fn list_instances(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/api/instances")
        .await?
        .response?;

    Ok(())
}

fn webrtc_scenario() -> Scenario {
    scenario!("WebRTC")
        .register_transaction(transaction!(join_room))
        .register_transaction(transaction!(send_message))
}

async fn join_room(user: &mut GooseUser) -> TransactionResult {
    let payload = serde_json::json!({
        "room_id": "test-room",
        "user_id": "test-user"
    });

    let _response = user
        .post_json("/webrtc/rooms/join", &payload)
        .await?
        .response?;

    Ok(())
}

async fn send_message(user: &mut GooseUser) -> TransactionResult {
    let payload = serde_json::json!({
        "room_id": "test-room",
        "message": "test message"
    });

    let _response = user
        .post_json("/webrtc/messages", &payload)
        .await?
        .response?;

    Ok(())
}
