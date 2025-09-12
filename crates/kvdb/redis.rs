use anyhow::Result;
use bb8_redis::{
    RedisConnectionManager,
    bb8::Pool,
    redis::{AsyncCommands as _, cmd},
};
use bon::Builder;
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::time::Duration;
use tracing::info;

use crate::{KvdbTrait, serde_util};

#[derive(Builder)]
pub struct RedisKvdbConfig {
    url: String,
    connection_timeout: Duration,
    max_size: u32,
    min_idle: u32,
    max_lifetime: Option<Duration>,
    idle_timeout: Option<Duration>,
}

pub struct RedisKvdb {
    pool: Pool<RedisConnectionManager>,
}

impl Debug for RedisKvdb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisKvdb").finish()
    }
}

impl RedisKvdb {
    pub async fn try_new(config: RedisKvdbConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(&*config.url)?;
        let pool = Pool::builder()
            .connection_timeout(config.connection_timeout)
            .max_size(config.max_size)
            .min_idle(config.min_idle)
            .max_lifetime(config.max_lifetime)
            .idle_timeout(config.idle_timeout)
            .build(manager)
            .await?;
        let instance = Self { pool: pool.clone() };
        instance.print_info().await?;
        Ok(instance)
    }
}

impl RedisKvdb {
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
impl KvdbTrait for RedisKvdb {
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

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let mut cursor: u64 = 0;
        loop {
            // 使用 SCAN 匹配 key
            let (next_cursor, keys): (u64, Vec<String>) = cmd("SCAN")
                .cursor_arg(cursor)
                .arg("MATCH")
                .arg(format!("{}*", prefix))
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut *conn)
                .await?;

            if !keys.is_empty() {
                // 批量删除
                let _: () = conn.del(keys).await?;
            }

            if next_cursor == 0 {
                break;
            }
            cursor = next_cursor;
        }

        Ok(())
    }

    async fn delete_expired(&self) -> Result<()> {
        Ok(())
    }
}
