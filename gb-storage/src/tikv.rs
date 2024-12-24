use gb_core::{Result, Error};
use tikv_client::{RawClient, Config, KvPair};
use tracing::{error, instrument};

pub struct TiKVStorage {
    client: RawClient,
}

impl TiKVStorage {
    pub async fn new(pd_endpoints: Vec<String>) -> Result<Self> {
        let config = Config::default();
        let client = RawClient::new(pd_endpoints)
            .await
            .map_err(|e| Error::internal(format!("TiKV error: {}", e)))?;

        Ok(Self { client })
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.client
            .get(key.to_vec())
            .await
            .map_err(|e| {
                error!("TiKV get error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.client
            .put(key.to_vec(), value.to_vec())
            .await
            .map_err(|e| {
                error!("TiKV put error: {}", e);
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
    pub async fn batch_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<KvPair>> {
        self.client
            .batch_get(keys)
            .await
            .map_err(|e| {
                error!("TiKV batch get error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }

    #[instrument(skip(self))]
    pub async fn scan(&self, start: &[u8], end: &[u8], limit: u32) -> Result<Vec<KvPair>> {
        self.client
            .scan(start.to_vec()..end.to_vec(), limit)
            .await
            .map_err(|e| {
                error!("TiKV scan error: {}", e);
                Error::internal(format!("TiKV error: {}", e))
            })
    }
}