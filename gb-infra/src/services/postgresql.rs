use crate::manager::Service;
use std::env;
use std::process::Command;
use std::collections::HashMap;
use dotenv::dotenv;

pub struct PostgreSQL {
    env_vars: HashMap<String, String>,
    process: Option<std::process::Child>,
}

impl PostgreSQL {
    pub fn new() -> Self {
        dotenv().ok();
        let env_vars = vec![
            "POSTGRES_DATA_DIR",
            "POSTGRES_PORT",
            "POSTGRES_USER",
            "POSTGRES_PASSWORD",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect();

        PostgreSQL {
            env_vars,
            process: None,
        }
    }
}

impl Service for PostgreSQL {
    fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("PostgreSQL is already running.".to_string());
        }

        // Initialize PostgreSQL data directory if it doesn't exist
        let data_dir = self.env_vars.get("POSTGRES_DATA_DIR").unwrap();
        if !std::path::Path::new(data_dir).exists() {
            Command::new("sudo")
                .arg("-u")
                .arg("postgres")
                .arg("/usr/lib/postgresql/14/bin/initdb")
                .arg("-D")
                .arg(data_dir)
                .status()
                .map_err(|e| e.to_string())?;
        }

        // Start PostgreSQL
        let mut command = Command::new("sudo");
        command
            .arg("-u")
            .arg("postgres")
            .arg("/usr/lib/postgresql/14/bin/pg_ctl")
            .arg("start")
            .arg("-D")
            .arg(data_dir);

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

        // Stop PostgreSQL
        let data_dir = self.env_vars.get("POSTGRES_DATA_DIR").unwrap();
        Command::new("sudo")
            .arg("-u")
            .arg("postgres")
            .arg("/usr/lib/postgresql/14/bin/pg_ctl")
            .arg("stop")
            .arg("-D")
            .arg(data_dir)
            .status()
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}