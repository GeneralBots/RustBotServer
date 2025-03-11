use gb_testing::chaos::ChaosTest;

#[tokio::test]
async fn test_kubernetes_chaos() -> anyhow::Result<()> {
    let chaos_test = ChaosTest::new().await?;
    
    
    chaos_test.network_partition().await?;
    chaos_test.resource_exhaustion().await?;
    
    Ok(())
}
