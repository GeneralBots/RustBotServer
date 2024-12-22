use async_trait::async_trait;
use gb_core::{
    models::*,
    traits::*,
    Result, Error,
};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{instrument, error};

pub struct PostgresCustomerRepository {
    pool: PgPool,
}

impl PostgresCustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for PostgresCustomerRepository {
    #[instrument(skip(self))]
    async fn create(&self, customer: &Customer) -> Result<Customer> {
        let record = sqlx::query_as!(
            Customer,
            r#"
            INSERT INTO customers (id, name, subscription_tier, status, max_instances, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            customer.id,
            customer.name,
            customer.subscription_tier,
            customer.status,
            customer.max_instances,
            customer.metadata as _,
            customer.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create customer: {}", e);
            Error::Database(e)
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn get(&self, id: Uuid) -> Result<Customer> {
        let record = sqlx::query_as!(
            Customer,
            r#"
            SELECT * FROM customers WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Customer {} not found", id)),
            e => Error::Database(e),
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn update(&self, customer: &Customer) -> Result<Customer> {
        let record = sqlx::query_as!(
            Customer,
            r#"
            UPDATE customers
            SET name = $1, subscription_tier = $2, status = $3, max_instances = $4, metadata = $5
            WHERE id = $6
            RETURNING *
            "#,
            customer.name,
            customer.subscription_tier,
            customer.status,
            customer.max_instances,
            customer.metadata as _,
            customer.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Customer {} not found", customer.id)),
            e => Error::Database(e),
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM customers WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Customer {} not found", id)),
            e => Error::Database(e),
        })?;

        Ok(())
    }
}

pub struct PostgresInstanceRepository {
    pool: PgPool,
}

impl PostgresInstanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InstanceRepository for PostgresInstanceRepository {
    #[instrument(skip(self))]
    async fn create(&self, instance: &Instance) -> Result<Instance> {
        let record = sqlx::query_as!(
            Instance,
            r#"
            INSERT INTO instances (id, customer_id, name, status, shard_id, region, config, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            instance.id,
            instance.customer_id,
            instance.name,
            instance.status,
            instance.shard_id,
            instance.region,
            instance.config as _,
            instance.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create instance: {}", e);
            Error::Database(e)
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn get(&self, id: Uuid) -> Result<Instance> {
        let record = sqlx::query_as!(
            Instance,
            r#"
            SELECT * FROM instances WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Instance {} not found", id)),
            e => Error::Database(e),
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn get_by_customer(&self, customer_id: Uuid) -> Result<Vec<Instance>> {
        let records = sqlx::query_as!(
            Instance,
            r#"
            SELECT * FROM instances WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(records)
    }

    #[instrument(skip(self))]
    async fn get_by_shard(&self, shard_id: i32) -> Result<Vec<Instance>> {
        let records = sqlx::query_as!(
            Instance,
            r#"
            SELECT * FROM instances WHERE shard_id = $1
            "#,
            shard_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(records)
    }

    #[instrument(skip(self))]
    async fn update(&self, instance: &Instance) -> Result<Instance> {
        let record = sqlx::query_as!(
            Instance,
            r#"
            UPDATE instances
            SET name = $1, status = $2, shard_id = $3, region = $4, config = $5
            WHERE id = $6
            RETURNING *
            "#,
            instance.name,
            instance.status,
            instance.shard_id,
            instance.region,
            instance.config as _,
            instance.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Instance {} not found", instance.id)),
            e => Error::Database(e),
        })?;

        Ok(record)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM instances WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("Instance {} not found", id)),
            e => Error::Database(e),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use sqlx::postgres::PgPoolOptions;

    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/gb_test".to_string());
            
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[fixture]
    fn customer() -> Customer {
        Customer::new(
            "Test Corp".to_string(),
            "enterprise".to_string(),
            10,
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_customer_crud(customer: Customer) {
        let pool = create_test_pool().await;
        let repo = PostgresCustomerRepository::new(pool);

        // Create
        let created = repo.create(&customer).await.unwrap();
        assert_eq!(created.name, customer.name);

        // Get
        let retrieved = repo.get(created.id).await.unwrap();
        assert_eq!(retrieved.id, created.id);

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Updated Corp".to_string();
        let updated = repo.update(&updated).await.unwrap();
        assert_eq!(updated.name, "Updated Corp");

        // Delete
        repo.delete(updated.id).await.unwrap();
        assert!(repo.get(updated.id).await.is_err());
    }
}
