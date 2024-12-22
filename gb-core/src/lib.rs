pub mod models;
pub mod traits;
pub mod errors;

pub use errors::{Error, Result};
pub use models::*;
pub use traits::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn customer() -> Customer {
        Customer::new(
            "Test Corp".to_string(),
            "enterprise".to_string(),
            10,
        )
    }

    #[rstest]
    fn test_customer_fixture(customer: Customer) {
        assert_eq!(customer.name, "Test Corp");
        assert_eq!(customer.subscription_tier, "enterprise");
        assert_eq!(customer.max_instances, 10);
        assert_eq!(customer.status, "active");
    }
}
