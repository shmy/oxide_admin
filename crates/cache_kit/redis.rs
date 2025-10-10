use std::time::Duration;

use crate::{CacheTrait, error::Result, serde_util};
use crate::{IterItem, JsonItem};
use bb8_redis::redis::{AsyncCommands as _, cmd, pipe};
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

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let items = self.iter_prefix(prefix).await?;
        let mut conn = self.pool.get().await?;
        if !items.is_empty() {
            let _: () = conn
                .del(items.into_iter().map(|item| item.key).collect::<Vec<_>>())
                .await?;
        }

        Ok(())
    }

    async fn iter_prefix(&self, prefix: &str) -> Result<Vec<IterItem>> {
        let mut conn = self.pool.get().await?;
        let mut cursor: u64 = 0;
        let mut all_items = Vec::new();

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = cmd("SCAN")
                .cursor_arg(cursor)
                .arg("MATCH")
                .arg(format!("{}*", prefix))
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut *conn)
                .await?;

            if !keys.is_empty() {
                let mut pipe = pipe();
                for k in &keys {
                    pipe.pttl(k);
                }
                let ttls: Vec<i64> = pipe.query_async(&mut *conn).await?;
                for (k, ttl) in keys.into_iter().zip(ttls.into_iter()) {
                    let expired_at = if ttl > 0 {
                        Some(((chrono::Utc::now().timestamp_millis() + ttl) / 1000) as u64)
                    } else {
                        None
                    };
                    all_items.push(IterItem { key: k, expired_at });
                }
            }

            if next_cursor == 0 {
                break;
            }
            cursor = next_cursor;
        }

        Ok(all_items)
    }

    async fn get_raw_string(&self, key: &str) -> Option<JsonItem> {
        let mut conn = self.pool.get().await.ok()?;
        let results: (Option<Vec<u8>>, i64) = pipe()
            .get(key)
            .ttl(key)
            .query_async(&mut *conn)
            .await
            .ok()?;

        let (value, ttl) = results;
        let expired_at = if ttl > 0 {
            Some((chrono::Utc::now().timestamp() + ttl) as u64)
        } else {
            None
        };

        value.and_then(|value| {
            serde_util::cbor_decode::<serde_json::Value>(&value)
                .ok()
                .map(|value| JsonItem { value, expired_at })
        })
    }
}
