use std::time::Duration;

use crate::{CacheTrait, error::Result, serde_util};
use bb8_redis::redis::AsyncCommands as _;
use bb8_redis::{RedisConnectionManager, bb8::Pool};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone, Debug)]
pub struct RedisCacheImpl {
    pool: Pool<RedisConnectionManager>,
}

impl RedisCacheImpl {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }
}

impl CacheTrait for RedisCacheImpl {
    async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let mut conn = self.pool.get().await.ok()?;
        conn.get(key)
            .await
            .ok()
            .and_then(|v: Vec<u8>| serde_util::cbor_decode(&v).ok())
    }

    async fn insert<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let value = serde_util::cbor_encode(&value)?;
        let mut conn = self.pool.get().await?;
        conn.set::<_, Vec<u8>, ()>(key, value).await?;
        Ok(())
    }

    async fn insert_with_ttl<T>(&self, key: &str, value: T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let value = serde_util::cbor_encode(&value)?;
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, Vec<u8>, ()>(key, value, ttl.as_secs())
            .await?;
        Ok(())
    }
}
