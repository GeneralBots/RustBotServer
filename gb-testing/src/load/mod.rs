use goose::goose::TransactionError;

use goose::prelude::*;

fn get_default_name() -> &'static str {
    "default"
}

pub struct LoadTestConfig {
    pub users: usize,
    pub ramp_up: usize,
    pub port: u16,
}

pub struct LoadTest {
    config: LoadTestConfig,
}

impl LoadTest {
    pub fn new(config: LoadTestConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut goose = GooseAttack::initialize()?;

        goose
            .set_default(GooseDefault::Host, &format!("http://localhost:{}", self.config.port).as_str())?
            .set_users(self.config.users)?
            .set_startup_time(self.config.ramp_up)?;

        Ok(())
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
        .response
        .map_err(|e| Box::new(TransactionError::RequestError(e.to_string())))?;

    Ok(())
}

async fn logout(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .post("/auth/logout")
        .await?
        .response
        .map_err(|e| Box::new(TransactionError::RequestError(e.to_string())))?;

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
        .response
        .map_err(|e| Box::new(TransactionError::RequestFailed(e.to_string())))?;

    Ok(())
}

async fn list_instances(user: &mut GooseUser) -> TransactionResult {
    let _response = user
        .get("/api/instances")
        .await?
        .response
        .map_err(|e| Box::new(TransactionError::RequestError(e.to_string())))?;

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
        .response
        .map_err(|e| Box::new(TransactionError::RequestError(e.to_string())))?;

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
        .response
        .map_err(|e| Box::new(TransactionError::RequestError(e.to_string())))?;

    Ok(())
}

impl From<reqwest::Error> for TransactionError {
    fn from(error: reqwest::Error) -> Self {
        TransactionError::RequestError(error.to_string())
    }
}
