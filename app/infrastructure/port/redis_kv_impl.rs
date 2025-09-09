use std::time::Duration;

use anyhow::Result;
use bb8_redis::{RedisConnectionManager, bb8::Pool, redis::AsyncTypedCommands};
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use crate::shared::{kv::KvTrait, serde_util};

#[derive(Debug, Clone)]
pub struct RedisKVImpl {
    pool: Pool<RedisConnectionManager>,
}

impl RedisKVImpl {
    pub async fn try_new(url: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(Self { pool })
    }
}

impl KvTrait for RedisKVImpl {
    async fn get<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T> {
        let mut conn = self.pool.get().await?;
        if let Some(value) = conn.get(key).await? {
            let val: T = serde_util::rmp_decode(value.as_bytes());
            return Ok(val);
        }
        anyhow::bail!("Key not found");
    }

    async fn set_with_ex<T: Serialize>(
        &self,
        key: &str,
        value: T,
        duration: Duration,
    ) -> Result<()> {
        let value = serde_util::rmp_encode(&value);
        let mut conn = self.pool.get().await?;
        conn.set_ex(key, value, duration.as_secs()).await?;
        Ok(())
    }

    async fn set_with_ex_at<T: Serialize>(
        &self,
        key: &str,
        value: T,
        expires_at: i64,
    ) -> Result<()> {
        let value = serde_util::rmp_encode(&value);
        let now = Utc::now();
        let duration_secs = (expires_at - now.timestamp()).max(0) as u64;
        let mut conn = self.pool.get().await?;
        conn.set_ex(key, value, duration_secs).await?;
        Ok(())
    }

    async fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let value = serde_util::rmp_encode(&value);
        let mut conn = self.pool.get().await?;
        conn.set(key, value).await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        conn.del(key).await?;
        Ok(())
    }

    async fn delete_expired(&self) -> Result<()> {
        Ok(())
    }
}
