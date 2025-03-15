use crate::manager::Service;
use std::env;
use std::process::Command;
use std::collections::HashMap;
use dotenv::dotenv;

pub struct NGINX {
    env_vars: HashMap<String, String>,
    process: Option<std::process::Child>,
}

impl NGINX {
    pub fn new() -> Self {
        dotenv().ok();
        let env_vars = vec![
            "NGINX_ERROR_LOG",
            "NGINX_ACCESS_LOG",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect();

        NGINX {
            env_vars,
            process: None,
        }
    }
}

impl Service for NGINX {
    fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("NGINX is already running.".to_string());
        }

        // Configure NGINX logs
        let error_log = self.env_vars.get("NGINX_ERROR_LOG").unwrap();
        let access_log = self.env_vars.get("NGINX_ACCESS_LOG").unwrap();

        // Update NGINX configuration
        let nginx_conf = format!(
            r#"
error_log {} debug;
access_log {};
events {{}}
http {{
    server {{
        listen 80;
        server_name localhost;
        location / {{
            root /var/www/html;
        }}
    }}
}}
"#,
            error_log, access_log
        );

        // Write the configuration to /etc/nginx/nginx.conf
        std::fs::write("/etc/nginx/nginx.conf", nginx_conf).map_err(|e| e.to_string())?;

        // Start NGINX
        let mut command = Command::new("nginx");
        self.process = Some(command.spawn().map_err(|e| e.to_string())?);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
        }

        // Stop NGINX
        Command::new("nginx")
            .arg("-s")
            .arg("stop")
            .status()
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}