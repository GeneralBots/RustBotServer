pub mod errors;
pub mod models;
pub mod traits;
pub use errors::{Error, ErrorKind, Result};



#[derive(Clone)]
struct AppState {
    db: PgPool,
    redis: RedisClient,
    storage: MinioClient,
    message_processor: MessageProcessor,
    customer: PostgresCustomerRepository,
}


#[cfg(test)]
mod tests {
    use crate::models::{Customer, SubscriptionTier};
    use rstest::*;

#[fixture]
    fn customer() -> Customer {
        Customer::new(
            "Test Corp".to_string(),
            "test@example.com".to_string(),
            SubscriptionTier::Enterprise,
            10,
        )
    }

    #[rstest]
    fn test_customer_fixture(customer: Customer) {
        assert_eq!(customer.name, "Test Corp");
        assert_eq!(customer.email, "test@example.com"); 
        
        assert_eq!(customer.max_instances, 10);

    }
 }
