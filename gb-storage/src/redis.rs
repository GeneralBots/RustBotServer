use async_trait::async_trait;
use gb_core::{Result, Error};
use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{instrument, error};

pub struct RedisCache {
    client: Client,
    default_ttl: Duration,
}

impl RedisCache {
    pub fn new(url: &str, default_ttl: Duration) -> Result<Self> {
        let client = Client::open(url).map_err(|e| Error::Redis(e))?;
        Ok(Self {
            client,
            default_ttl,
        })
    }

    #[instrument(skip(self, value))]
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(Error::Redis)?;

        let serialized = serde_json::to_string(value)
            .map_err(|e| Error::internal(format!("Serialization error: {}", e)))?;

        conn.set_ex(key, serialized, self.default_ttl.as_secs() as usize)
            .await
            .map_err(|e| {
                error!("Redis set error: {}", e);
                Error::Redis(e)
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(Error::Redis)?;

        let value: Option<String> = conn.get(key)
            .await
            .map_err(Error::Redis)?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| Error::internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(Error::Redis)?;

        conn.del(key)
            .await
            .map_err(|e| {
                error!("Redis delete error: {}", e);
                Error::Redis(e)
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn increment(&self, key: &str) -> Result<i64> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(Error::Redis)?;

        conn.incr(key, 1)
            .await
            .map_err(|e| {
                error!("Redis increment error: {}", e);
                Error::Redis(e)
            })
    }

    #[instrument(skip(self))]
    pub async fn set_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(Error::Redis)?;

        let serialized = serde_json::to_string(value)
            .map_err(|e| Error::internal(format!("Serialization error: {}", e)))?;

        conn.set_ex(key, serialized, ttl.as_secs() as usize)
            .await
            .map_err(|e| {
                error!("Redis set error: {}", e);
                Error::Redis(e)
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        field: String,
    }

    #[tokio::test]
    async fn test_redis_cache() {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

        let cache = RedisCache::new(&redis_url, Duration::from_secs(60)).unwrap();

        // Test set and get
        let test_value = TestStruct {
            field: "test".to_string(),
        };

        cache.set("test_key", &test_value).await.unwrap();
        let retrieved: Option<TestStruct> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved.unwrap(), test_value);

        // Test delete
        cache.delete("test_key").await.unwrap();
        let deleted: Option<TestStruct> = cache.get("test_key").await.unwrap();
        assert!(deleted.is_none());

        // Test increment
        cache.set("counter", &0).await.unwrap();
        let count = cache.increment("counter").await.unwrap();
        assert_eq!(count, 1);

        // Test TTL
        cache.set_with_ttl("ttl_key", &test_value, Duration::from_secs(1)).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let expired: Option<TestStruct> = cache.get("ttl_key").await.unwrap();
        assert!(expired.is_none());
    }
}
