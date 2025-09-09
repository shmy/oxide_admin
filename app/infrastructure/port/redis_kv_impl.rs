use std::time::Duration;

use anyhow::Result;
use bb8_redis::{
    RedisConnectionManager,
    bb8::Pool,
    redis::{AsyncCommands as _, cmd},
};
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use tracing::info;

use crate::shared::{kv::KvTrait, serde_util};

#[derive(Debug, Clone)]
pub struct RedisKVImpl {
    pool: Pool<RedisConnectionManager>,
}

impl RedisKVImpl {
    pub async fn try_new(url: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(10))
            .build(manager)
            .await?;
        let instance = Self { pool };
        instance.print_info().await?;
        Ok(instance)
    }
}

impl RedisKVImpl {
    async fn print_info(&self) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let info: String = cmd("INFO").arg("server").query_async(&mut *conn).await?;
        let mut version = String::new();
        let mut mode = String::new();
        let mut os = String::new();
        let mut gcc = String::new();
        let mut bit = String::new();
        for line in info.lines() {
            if line.starts_with("redis_version:") {
                version.push_str(line.trim_start_matches("redis_version:"));
            }
            if line.starts_with("redis_mode:") {
                mode.push_str(line.trim_start_matches("redis_mode:"));
            }
            if line.starts_with("os:") {
                os.push_str(line.trim_start_matches("os:"));
            }
            if line.starts_with("gcc_version:") {
                gcc.push_str(line.trim_start_matches("gcc_version:"));
            }
            if line.starts_with("arch_bits:") {
                bit.push_str(line.trim_start_matches("arch_bits:"));
            }
        }
        info!(
            "Redis {} ({}) on {}, compiled by gcc ({}) {}-bit",
            version, mode, os, gcc, bit
        );
        Ok(())
    }
}
impl KvTrait for RedisKVImpl {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.pool.get().await.ok()?;
        let data: Vec<u8> = conn.get(key).await.ok()?;
        serde_util::rmp_decode::<T>(&data).ok()
    }

    async fn set_with_ex<T: Serialize>(
        &self,
        key: &str,
        value: T,
        duration: Duration,
    ) -> Result<()> {
        let value = serde_util::rmp_encode(&value)?;
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, Vec<u8>, ()>(key, value, duration.as_secs())
            .await?;
        Ok(())
    }

    async fn set_with_ex_at<T: Serialize>(
        &self,
        key: &str,
        value: T,
        expires_at: i64,
    ) -> Result<()> {
        let value = serde_util::rmp_encode(&value)?;
        let now = Utc::now();
        let duration_secs = (expires_at - now.timestamp()).max(0) as u64;
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, Vec<u8>, ()>(key, value, duration_secs)
            .await?;
        Ok(())
    }

    async fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let value = serde_util::rmp_encode(&value)?;
        let mut conn = self.pool.get().await?;
        conn.set::<_, Vec<u8>, ()>(key, value).await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    async fn delete_expired(&self) -> Result<()> {
        Ok(())
    }
}
