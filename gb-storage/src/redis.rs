use gb_core::{Result, Error};
use redis::{Client, Commands};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::instrument;

pub struct RedisStorage {
    client: Client,
}

impl RedisStorage {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

            Ok(Self { client })
        }
    
        
        #[allow(dependency_on_unit_never_type_fallback)]
        #[instrument(skip(self))]
    pub async fn set<T: Serialize + std::fmt::Debug>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;
        
        let serialized = serde_json::to_string(value)
            .map_err(|e| Error::internal(format!("Serialization error: {}", e)))?;

        conn.set(key, serialized)
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.client.get_connection()
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        let value: Option<String> = conn.get(key)
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| Error::internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None)
        }
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        conn.del(key)
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    #[instrument(skip(self))]
    pub async fn set_with_ttl<T: Serialize + std::fmt::Debug>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        let serialized = serde_json::to_string(value)
            .map_err(|e| Error::internal(format!("Serialization error: {}", e)))?;

        redis::pipe()
            .set(key, serialized)
            .expire(key, ttl.as_secs() as i64)
            .query(&mut conn)
            .map_err(|e| Error::internal(format!("Redis error: {}", e)))?;

        Ok(())
    }
}