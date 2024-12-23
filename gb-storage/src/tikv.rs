use async_trait::async_trait;
use gb_core::{Result, Error};
use tikv_client::{Config, RawClient, Value};
use tracing::{instrument, error};

pub struct TiKVStorage {
    client: RawClient,
}

impl TiKVStorage {
    pub async fn new(pd_endpoints: Vec<String>) -> Result<Self> {
        let config = Config::default();
        let client = RawClient::new(pd_endpoints, config)
            .await
            .map_err(|e| Error::internal(format!("TiKV client error: {}", e)))?;

        Ok(Self { client })
    }

    #[instrument(skip(self, value))]
    pub async fn put(&self, key: &[u8], value: Value) -> Result<()> {
        self.client
            .put(key.to_vec(), value)
            .await
            .map_err(|e| {
                error!("TiKV put error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &[u8]) -> Result<Option<Value>> {
        self.client
            .get(key.to_vec())
            .await
            .map_err(|e| {
                error!("TiKV get error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, key: &[u8]) -> Result<()> {
        self.client
            .delete(key.to_vec())
            .await
            .map_err(|e| {
                error!("TiKV delete error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn batch_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<KVPair>> {
        self.client
            .batch_get(keys)
            .await
            .map_err(|e| {
                error!("TiKV batch get error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn scan(&self, start: &[u8], end: &[u8], limit: u32) -> Result<Vec<KVPair>> {
        self.client
            .scan(start.to_vec()..end.to_vec(), limit)
            .await
            .map_err(|e| {
                error!("TiKV scan error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }
}

#[derive(Debug, Clone)]
pub struct KVPair {
    pub key: Vec<u8>,
    pub value: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tikv_client::Value;

    #[tokio::test]
    async fn test_tikv_storage() {
        let pd_endpoints = vec!["127.0.0.1:2379".to_string()];
        let storage = TiKVStorage::new(pd_endpoints).await.unwrap();

        // Test put and get
        let key = b"test_key";
        let value = Value::from(b"test_value".to_vec());
        storage.put(key, value.clone()).await.unwrap();

        let retrieved = storage.get(key).await.unwrap();
        assert_eq!(retrieved.unwrap(), value);

        // Test delete
        storage.delete(key).await.unwrap();
        let deleted = storage.get(key).await.unwrap();
        assert!(deleted.is_none());

        // Test batch operations
        let pairs = vec![
            (b"key1".to_vec(), Value::from(b"value1".to_vec())),
            (b"key2".to_vec(), Value::from(b"value2".to_vec())),
        ];

        for (key, value) in pairs.clone() {
            storage.put(&key, value).await.unwrap();
        }

        let keys: Vec<Vec<u8>> = pairs.iter().map(|(k, _)| k.clone()).collect();
        let retrieved = storage.batch_get(keys).await.unwrap();
        assert_eq!(retrieved.len(), pairs.len());

        // Test scan
        let scanned = storage.scan(b"key", b"key3", 10).await.unwrap();
        assert_eq!(scanned.len(), 2);

        // Cleanup
        for (key, _) in pairs {
            storage.delete(&key).await.unwrap();
        }
    }
}
