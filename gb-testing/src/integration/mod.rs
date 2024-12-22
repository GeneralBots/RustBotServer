use async_trait::async_trait;
use sqlx::PgPool;
use testcontainers::clients::Cli;

pub struct IntegrationTest {
    pub docker: Cli,
    pub db_pool: PgPool,
}

#[async_trait]
pub trait IntegrationTestCase {
    async fn setup(&mut self) -> anyhow::Result<()>;
    async fn execute(&self) -> anyhow::Result<()>;
    async fn teardown(&mut self) -> anyhow::Result<()>;
}

//pub struct TestEnvironment {
    //pub postgres: testcontainers::Container<'static, testcontainers::images::postgres::Postgres>,
    //pub redis: testcontainers::Container<'static, testcontainers::images::redis::Redis>,
    // pub kafka: testcontainers::Container<'static, testcontainers::images::kafka::Kafka>,
//

impl TestEnvironment {
    pub async fn new() -> anyhow::Result<Self> {
        let docker = Cli::default();
        
        // Start PostgreSQL
        let postgres = docker.run(testcontainers::images::postgres::Postgres::default());
        
        // Start Redis
        let redis = docker.run(testcontainers::images::redis::Redis::default());
        
        // Start Kafka
        let kafka = docker.run(testcontainers::images::kafka::Kafka::default());

        Ok(Self {
            postgres,
            redis,
            kafka,
        })
    }
}
