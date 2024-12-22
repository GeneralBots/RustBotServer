pub mod postgres;
pub mod redis;
pub mod tikv;

pub use postgres::{PostgresCustomerRepository, PostgresInstanceRepository};
pub use redis::RedisCache;
pub use tikv::TiKVStorage;

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::models::Customer;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/gb_test".to_string());
            
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        // Run migrations
        gb_migrations::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_storage_integration() {
        // Setup PostgreSQL
        let pool = setup_test_db().await;
        let customer_repo = PostgresCustomerRepository::new(pool.clone());

        // Setup Redis
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let cache = RedisCache::new(&redis_url, Duration::from_secs(60)).unwrap();

        // Create a customer
        let customer = Customer::new(
            "Integration Test Corp".to_string(),
            "enterprise".to_string(),
            10,
        );

        // Save to PostgreSQL
        let created = customer_repo.create(&customer).await.unwrap();

        // Cache in Redis
        cache.set(&format!("customer:{}", created.id), &created).await.unwrap();

        // Verify Redis cache
        let cached: Option<Customer> = cache.get(&format!("customer:{}", created.id)).await.unwrap();
        assert_eq!(cached.unwrap().id, created.id);

        // Cleanup
        customer_repo.delete(created.id).await.unwrap();
        cache.delete(&format!("customer:{}", created.id)).await.unwrap();
    }
}
