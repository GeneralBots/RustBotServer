use gb_core::{Result, Error};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use gb_core::models::Customer;

pub trait CustomerRepository {
    async fn create(&self, customer: Customer) -> Result<Customer>;
    async fn get(&self, id: Uuid) -> Result<Option<Customer>>;
    async fn update(&self, customer: Customer) -> Result<Customer>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

pub struct PostgresCustomerRepository {
    pool: Arc<PgPool>,
}

impl PostgresCustomerRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl CustomerRepository for PostgresCustomerRepository {
    async fn create(&self, customer: Customer) -> Result<Customer> {
        let result = sqlx::query_as!(
            Customer,
            r#"
            INSERT INTO customers (id, name, max_instances, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
            customer.id,
            customer.name,
            customer.max_instances as i32,
            customer.email,
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Customer>> {
        let result = sqlx::query_as!(
            Customer,
            r#"
            SELECT id, name, max_instances::int as "max_instances!: i32", 
                   email, created_at, updated_at
            FROM customers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, customer: Customer) -> Result<Customer> {
        let result = sqlx::query_as!(
            Customer,
            r#"
            UPDATE customers
            SET name = $2, max_instances = $3, email = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, max_instances::int as "max_instances!: i32", 
                      email, created_at, updated_at
            "#,
            customer.id,
            customer.name,
            customer.max_instances as i32,
            customer.email,
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM customers
            WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}