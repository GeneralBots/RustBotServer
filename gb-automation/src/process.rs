use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tokio::sync::Mutex;
use tracing::{error, instrument};
use uuid::Uuid;
use gb_core::{Error, Result};

#[derive(Debug)]
struct Process {
    id: Uuid,
    handle: Child,
}

pub struct ProcessAutomation {
    working_dir: PathBuf,
    processes: Mutex<Vec<Process>>,
}

impl ProcessAutomation {
    pub fn new(working_dir: impl AsRef<Path>) -> Self {
        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
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

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::internal(format!("Command failed: {}", error)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    pub async fn spawn(&self, command: &str, args: &[&str]) -> Result<Uuid> {
        let child = Command::new(command)
            .args(args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn process: {}", e)))?;

        let id = Uuid::new_v4();
        let mut processes = self.processes.lock().await;
        processes.push(Process { id, handle: child });
        
        Ok(id)
    }

    pub async fn kill(&self, id: Uuid) -> Result<()> {
        let mut processes = self.processes.lock().await;
        
        if let Some(index) = processes.iter().position(|p| p.id == id) {
            let mut process = processes.remove(index);
            process.handle.kill()
                .map_err(|e| Error::internal(format!("Failed to kill process: {}", e)))?;
            }
        Ok(())
    }

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
    use tempfile::tempdir;

    fn automation() -> ProcessAutomation {
        let dir = tempdir().unwrap();
        ProcessAutomation::new(dir.path())
    }

    #[tokio::test]
    async fn test_execute() -> Result<()> {
        let automation = automation();
        let output = automation.execute("echo", &["Hello, World!"]).await?;
        assert!(output.contains("Hello, World!"));
        Ok(())
    }

    #[tokio::test]
    async fn test_spawn_and_kill() -> Result<()> {
        let automation = automation();
        let id = automation.spawn("sleep", &["1"]).await?;
        automation.kill(id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cleanup() -> Result<()> {
        let automation = automation();
        automation.spawn("sleep", &["1"]).await?;
        automation.spawn("sleep", &["2"]).await?;
        automation.cleanup().await?;
        
        let processes = automation.processes.lock().await;
        assert!(processes.is_empty());
        
        Ok(())
}
}
