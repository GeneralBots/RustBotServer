use gb_core::{
    Result, Error,
    models::{Customer, CustomerStatus, SubscriptionTier},
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresCustomerRepository {
    pool: Arc<PgPool>,
}

impl PostgresCustomerRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, customer: Customer) -> Result<Customer> {
        let subscription_tier: String = customer.subscription_tier.clone().into();
        let status: String = customer.status.clone().into();

        let row = sqlx::query!(
            r#"
            INSERT INTO customers (
                id, name, email, max_instances, 
                subscription_tier, status, 
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            customer.id,
            customer.name,
            customer.email,
            customer.max_instances as i32,
            subscription_tier,
            status,
            customer.created_at,
            customer.updated_at,
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(Customer {
            id: row.id,
            name: row.name,
            email: row.email,
            max_instances: row.max_instances as u32,
            subscription_tier: SubscriptionTier::from(row.subscription_tier),
            status: CustomerStatus::from(row.status),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Customer>> {
        let row = sqlx::query!(
            r#"
            SELECT *
            FROM customers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(row.map(|row| Customer {
            id: row.id,
            name: row.name,
            email: row.email,
            max_instances: row.max_instances as u32,
            subscription_tier: SubscriptionTier::from(row.subscription_tier),
            status: CustomerStatus::from(row.status),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }
}