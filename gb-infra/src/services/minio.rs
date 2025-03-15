use crate::manager::Service;
use std::env;
use std::process::Command;
use std::collections::HashMap;
use dotenv::dotenv;

pub struct MinIO {
    env_vars: HashMap<String, String>,
    process: Option<std::process::Child>,
}

impl MinIO {
    pub fn new() -> Self {
        dotenv().ok();
        let env_vars = vec![
            "MINIO_ROOT_USER",
            "MINIO_ROOT_PASSWORD",
            "MINIO_VOLUMES",
            "MINIO_ADDRESS",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect();

        MinIO {
            env_vars,
            process: None,
        }
    }
}

impl Service for MinIO {
    fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("MinIO is already running.".to_string());
        }

        let mut command = Command::new("/opt/gbo/bin/minio");
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
