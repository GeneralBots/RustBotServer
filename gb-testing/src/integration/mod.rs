use async_trait::async_trait;
use sqlx::PgPool;

pub struct IntegrationTest {
    pub db_pool: PgPool,
}

#[async_trait]
pub trait IntegrationTestCase {
    async fn setup(&mut self) -> anyhow::Result<()>;
    async fn execute(&self) -> anyhow::Result<()>;
    async fn teardown(&mut self) -> anyhow::Result<()>;
}