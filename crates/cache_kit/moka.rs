use std::time::Duration;

use moka::future::Cache;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{CacheTrait, error::Result, serde_util};

#[derive(Clone, Serialize, Deserialize)]
struct CacheItem {
    data: Vec<u8>,
    expires_at: u64,
}

impl CacheItem {
    fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        now_timestamp() >= self.expires_at
    }
}

fn now_timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

#[derive(Clone)]
pub struct MokaCacheImpl {
    cache: Cache<String, CacheItem>,
}

impl MokaCacheImpl {
    pub fn new(max_capacity: u64, time_to_idle: Duration) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_idle(time_to_idle)
                .build(),
        }
    }

    async fn insert_inner<T>(&self, key: &str, value: T, expires_at: u64) -> Result<()>
    where
        T: Serialize,
    {
        if let Ok(bytes) = serde_util::cbor_encode(&value) {
            self.cache
                .insert(
                    key.to_string(),
                    CacheItem {
                        data: bytes,
                        expires_at,
                    },
                )
                .await;
        }
        Ok(())
    }
}

impl CacheTrait for MokaCacheImpl {
    async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.cache.get(key).await.and_then(|item| {
            if item.is_expired() {
                return None;
            }
            serde_util::cbor_decode::<T>(&item.data).ok()
        })
    }

    async fn insert<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        self.insert_inner(key, value, 0).await
    }

    async fn insert_with_ttl<T>(&self, key: &str, value: T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let expires_at = now_timestamp() + ttl.as_secs();
        self.insert_inner(key, value, expires_at).await
    }
}
