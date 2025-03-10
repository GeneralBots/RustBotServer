use gb_testing::load::{LoadTest, LoadTestConfig};
use std::time::Duration;

#[tokio::test]
async fn test_auth_load() -> anyhow::Result<()> {
    let config = LoadTestConfig {
        users: 100,
        duration: Duration::from_secs(300),
        ramp_up: Duration::from_secs(60),
        scenarios: vec!["auth".to_string()],
    };

    // let load_test = LoadTest::new(config);
    // let report = load_test.run().await?;
    
    // report.save_json("test-reports/auth-load-test.json")?;
    // report.save_html("test-reports/auth-load-test.html")?;

    Ok(())
}
