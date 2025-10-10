use std::{fmt::Debug, time::Duration};

use moka::future::Cache;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{CacheTrait, JsonItem, error::Result, serde_util};

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

impl Debug for MokaCacheImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MokaCacheImpl").finish()
    }
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

    async fn iter_prefix(&self, prefix: &str) -> Result<Vec<crate::IterItem>> {
        let mut result = Vec::new();
        let entries = self.cache.iter();
        for (key, item) in entries {
            if key.starts_with(prefix) && !item.is_expired() {
                result.push(crate::IterItem {
                    key: key.to_string(),
                    expired_at: if item.expires_at == 0 {
                        None
                    } else {
                        Some(item.expires_at)
                    },
                });
            }
        }
        Ok(result)
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let keys: Vec<String> = self
            .cache
            .iter()
            .filter_map(|(key, _)| {
                if key.starts_with(prefix) {
                    Some(key.to_string())
                } else {
                    None
                }
            })
            .collect();
        for key in keys {
            self.cache.invalidate(&key).await;
        }
        Ok(())
    }

    async fn get_raw_string(&self, key: &str) -> Option<JsonItem> {
        self.cache.get(key).await.and_then(|item| {
            if item.is_expired() {
                return None;
            }
            serde_util::cbor_decode::<Value>(&item.data)
                .ok()
                .map(|value| JsonItem {
                    value,
                    expired_at: if item.expires_at == 0 {
                        None
                    } else {
                        Some(item.expires_at)
                    },
                })
        })
    }
}
