// or wherever SubscriptionTier is defined

impl From<SubscriptionTier> for String {
    fn from(tier: SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Free => "free".to_string(),
            SubscriptionTier::Pro => "pro".to_string(),
            SubscriptionTier::Enterprise => "enterprise".to_string(),
        }
    }
}

impl From<CustomerStatus> for String {
    fn from(status: CustomerStatus) -> Self {
        match status {
            CustomerStatus::Active => "active".to_string(),
            CustomerStatus::Inactive => "inactive".to_string(),
            CustomerStatus::Suspended => "suspended".to_string(),
        }
    }
}
