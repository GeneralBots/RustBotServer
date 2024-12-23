use gb_core::{Result, Error};
use std::{
    process::{Command, Stdio},
    path::PathBuf,
};
use tokio::sync::Mutex;
use tracing::{instrument, error};
use uuid::Uuid;

pub struct ProcessAutomation {
    working_dir: PathBuf,
    processes: Mutex<Vec<Process>>,
}

pub struct Process {
    id: Uuid,
    handle: std::process::Child,
}

impl ProcessAutomation {
    pub fn new<P: Into<PathBuf>>(working_dir: P) -> Self {
        Self {
            working_dir: working_dir.into(),
            processes: Mutex::new(Vec::new()),
        }
    }

    #[instrument(skip(self, command))]
    pub async fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command)
            .args(args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Error::internal(format!("Failed to execute command: {}", e)))?;

            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::internal(format!("Command failed: {}", error)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    #[instrument(skip(self, command))]
    pub async fn spawn(&self, command: &str, args: &[&str]) -> Result<Uuid> {
        let child = Command::new(command)
            .args(args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn process: {}", e)))?;

        let process = Process {
            id: Uuid::new_v4(),
            handle: child,
        };

        let mut processes = self.processes.lock().await;
        processes.push(process);

        Ok(process.id)
    }

    #[instrument(skip(self))]
    pub async fn kill(&self, id: Uuid) -> Result<()> {
        let mut processes = self.processes.lock().await;
        
        if let Some(index) = processes.iter().position(|p| p.id == id) {
            let process = processes.remove(index);
            process.handle.kill()
                .map_err(|e| Error::internal(format!("Failed to kill process: {}", e)))?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cleanup(&self) -> Result<()> {
        let mut processes = self.processes.lock().await;
        
        for process in processes.iter_mut() {
            if let Err(e) = process.handle.kill() {
                error!("Failed to kill process {}: {}", process.id, e);
            }
        }

        processes.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::fs;
    use tempfile::tempdir;

    #[fixture]
    fn automation() -> ProcessAutomation {
        let dir = tempdir().unwrap();
        ProcessAutomation::new(dir.path())
    }

    #[rstest]
    #[tokio::test]
    async fn test_execute(automation: ProcessAutomation) -> Result<()> {
        let output = automation.execute("echo", &["Hello, World!"]).await?;
        assert!(output.contains("Hello, World!"));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_spawn_and_kill(automation: ProcessAutomation) -> Result<()> {
        let id = automation.spawn("sleep", &["1"]).await?;
        automation.kill(id).await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_cleanup(automation: ProcessAutomation) -> Result<()> {
        automation.spawn("sleep", &["1"]).await?;
        automation.spawn("sleep", &["2"]).await?;
        automation.cleanup().await?;
        
        let processes = automation.processes.lock().await;
        assert!(processes.is_empty());
        
        Ok(())
    }
}
