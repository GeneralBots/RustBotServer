use crate::manager::Service;
use std::env;
use std::process::Command;
use std::collections::HashMap;
use dotenv::dotenv;

pub struct Zitadel {
    env_vars: HashMap<String, String>,
    process: Option<std::process::Child>,
}

impl Zitadel {
    pub fn new() -> Self {
        dotenv().ok();
        let env_vars = vec![
            "ZITADEL_DEFAULTINSTANCE_INSTANCENAME",
            "ZITADEL_DEFAULTINSTANCE_ORG_NAME",
            "ZITADEL_DATABASE_POSTGRES_HOST",
            "ZITADEL_DATABASE_POSTGRES_PORT",
            "ZITADEL_DATABASE_POSTGRES_DATABASE",
            "ZITADEL_DATABASE_POSTGRES_USER_USERNAME",
            "ZITADEL_DATABASE_POSTGRES_USER_PASSWORD",
            "ZITADEL_DATABASE_POSTGRES_ADMIN_SSL_MODE",
            "ZITADEL_DATABASE_POSTGRES_USER_SSL_MODE",
            "ZITADEL_DATABASE_POSTGRES_ADMIN_USERNAME",
            "ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD",
            "ZITADEL_EXTERNALSECURE",
            "ZITADEL_MASTERKEY",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect();

        Zitadel {
            env_vars,
            process: None,
        }
    }
}

impl Service for Zitadel {
    fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("Zitadel is already running.".to_string());
        }

        let mut command = Command::new("/opt/gbo/bin/zitadel");
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
