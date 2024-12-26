use gb_core::{
    Result, Error,
    models::{Customer, CustomerStatus, SubscriptionTier},
};
use sqlx::{PgPool, Row, postgres::PgRow};
use std::sync::Arc;
use chrono::Utc;

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, customer: Customer) -> Result<Customer>;
    async fn get_customer_by_id(&self, id: &str) -> Result<Option<Customer>>;
    async fn update(&self, customer: Customer) -> Result<Customer>;
    async fn delete(&self, id: &str) -> Result<()>;
}

trait ToDbString {
    fn to_db_string(&self) -> String;
}

trait FromDbString: Sized {
    fn from_db_string(s: &str) -> Result<Self>;
}

impl ToDbString for SubscriptionTier {
    fn to_db_string(&self) -> String {
        match self {
            SubscriptionTier::Free => "free".to_string(),
            SubscriptionTier::Pro => "pro".to_string(),
            SubscriptionTier::Enterprise => "enterprise".to_string(),
        }
    }
}

impl ToDbString for CustomerStatus {
    fn to_db_string(&self) -> String {
        match self {
            CustomerStatus::Active => "active".to_string(),
            CustomerStatus::Inactive => "inactive".to_string(),
            CustomerStatus::Suspended => "suspended".to_string(),
        }
    }
}

impl FromDbString for SubscriptionTier {
    fn from_db_string(s: &str) -> Result<Self> {
        match s {
            "free" => Ok(SubscriptionTier::Free),
            "pro" => Ok(SubscriptionTier::Pro),
            "enterprise" => Ok(SubscriptionTier::Enterprise),
            _ => Err(Error::internal(format!("Invalid subscription tier: {}", s))),
        }
    }
}

impl FromDbString for CustomerStatus {
    fn from_db_string(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(CustomerStatus::Active),
            "inactive" => Ok(CustomerStatus::Inactive),
            "suspended" => Ok(CustomerStatus::Suspended),
            _ => Err(Error::internal(format!("Invalid customer status: {}", s))),
        }
    }
}

pub struct PostgresCustomerRepository {
    pool: Arc<PgPool>,
}

#[async_trait::async_trait]
impl CustomerRepository for PostgresCustomerRepository {
    async fn create(&self, customer: Customer) -> Result<Customer> {
        let subscription_tier = customer.subscription_tier.to_db_string();
        let status = customer.status.to_db_string();

        let row = sqlx::query(
            r#"
            INSERT INTO customers (
                id, name, email, 
                subscription_tier, status,
                created_at, updated_at,
                max_instances
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(&customer.id)
        .bind(&customer.name)
        .bind(&customer.email)
        .bind(&subscription_tier)
        .bind(&status)
        .bind(&customer.created_at)
        .bind(&customer.updated_at)
        .bind(customer.max_instances as i32)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Self::row_to_customer(&row).await
    }

    async fn get_customer_by_id(&self, id: &str) -> Result<Option<Customer>> {
        let maybe_row = sqlx::query(
            "SELECT * FROM customers WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        if let Some(row) = maybe_row {
            Ok(Some(Self::row_to_customer(&row).await?))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, customer: Customer) -> Result<Customer> {
        let subscription_tier = customer.subscription_tier.to_db_string();
        let status = customer.status.to_db_string();

        let row = sqlx::query(
            r#"
            UPDATE customers 
            SET name = $2, 
                email = $3, 
                subscription_tier = $4, 
                status = $5,
                updated_at = $6,
                max_instances = $7
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(&customer.id)
        .bind(&customer.name)
        .bind(&customer.email)
        .bind(&subscription_tier)
        .bind(&status)
        .bind(Utc::now())
        .bind(customer.max_instances as i32)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Self::row_to_customer(&row).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM customers WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}

impl PostgresCustomerRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn row_to_customer(row: &PgRow) -> Result<Customer> {
        Ok(Customer {
            id: row.try_get("id").map_err(|e| Error::internal(e.to_string()))?,
            name: row.try_get("name").map_err(|e| Error::internal(e.to_string()))?,
            email: row.try_get("email").map_err(|e| Error::internal(e.to_string()))?,
            subscription_tier: SubscriptionTier::from_db_string(
                row.try_get("subscription_tier").map_err(|e| Error::internal(e.to_string()))?
            )?,
            status: CustomerStatus::from_db_string(
                row.try_get("status").map_err(|e| Error::internal(e.to_string()))?
            )?,
            created_at: row.try_get("created_at").map_err(|e| Error::internal(e.to_string()))?,
            updated_at: row.try_get("updated_at").map_err(|e| Error::internal(e.to_string()))?,
            max_instances: {
                let value: i32 = row.try_get("max_instances")
                    .map_err(|e| Error::internal(e.to_string()))?;
                if value < 0 {
                    return Err(Error::internal("max_instances cannot be negative"));
                }
                value as u32
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[allow(dead_code)]
    fn create_test_customer() -> Customer {
        Customer {
            id: Uuid::new_v4(),
            name: "Test Customer".to_string(),
            email: "test@example.com".to_string(),
            subscription_tier: SubscriptionTier::Free,
            status: CustomerStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            max_instances: 1,
        }
    }

    // Add your tests here
    // Example:
    /*
    #[sqlx::test]
    async fn test_create_customer() {
        let pool = setup_test_db().await;
        let repo = PostgresCustomerRepository::new(Arc::new(pool));
        
        let customer = create_test_customer();
        let created = repo.create(customer.clone()).await.unwrap();
        
        assert_eq!(created.id, customer.id);
        assert_eq!(created.name, customer.name);
        // ... more assertions
    }
    */
}