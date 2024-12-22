use gb_testing::stress::StressTest;
use std::time::Duration;

#[tokio::test]
async fn test_system_stress() -> anyhow::Result<()> {
    let stress_test = StressTest::new(
        Duration::from_secs(1800),
        1000,
    );

    stress_test.run().await?;
    
    Ok(())
}
