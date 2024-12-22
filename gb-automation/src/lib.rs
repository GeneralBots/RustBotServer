pub mod web;
pub mod process;

pub use web::{WebAutomation, Element};
pub use process::ProcessAutomation;

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::Result;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_automation_integration() -> Result<()> {
        // Initialize automation components
        let web = WebAutomation::new().await?;
        let dir = tempdir()?;
        let process = ProcessAutomation::new(dir.path());

        // Test web automation
        let page = web.new_page().await?;
        web.navigate(&page, "https://example.com").await?;
        let screenshot = web.screenshot(&page, "test.png").await?;

        // Test process automation
        let output = process.execute("echo", &["Test output"]).await?;
        assert!(output.contains("Test output"));

        // Test process spawning and cleanup
        let id = process.spawn("sleep", &["1"]).await?;
        process.kill(id).await?;
        process.cleanup().await?;

        Ok(())
    }
}
