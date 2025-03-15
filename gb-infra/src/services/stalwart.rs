use crate::manager::Service;
use std::env;
use std::process::Command;
use std::collections::HashMap;
use dotenv::dotenv;

pub struct Stalwart {
    env_vars: HashMap<String, String>,
    process: Option<std::process::Child>,
}

impl Stalwart {
    pub fn new() -> Self {
        dotenv().ok();
        let env_vars = vec![
            "STALWART_LOG_LEVEL",
            "STALWART_OAUTH_PROVIDER",
            "STALWART_OAUTH_CLIENT_ID",
            "STALWART_OAUTH_CLIENT_SECRET",
            "STALWART_OAUTH_AUTHORIZATION_ENDPOINT",
            "STALWART_OAUTH_TOKEN_ENDPOINT",
            "STALWART_OAUTH_USERINFO_ENDPOINT",
            "STALWART_OAUTH_SCOPE",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect();

        Stalwart {
            env_vars,
            process: None,
        }
    }
}

impl Service for Stalwart {
    fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("Stalwart Mail is already running.".to_string());
        }

        let mut command = Command::new("/opt/gbo/bin/stalwart");
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        self.process = Some(command.spawn().map_err(|e| e.to_string())?);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
