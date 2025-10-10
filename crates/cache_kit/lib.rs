pub mod error;
#[cfg(feature = "moka")]
mod moka;
#[cfg(feature = "redis")]
mod redis;
mod serde_util;
pub use cache_kit_macros::cached_impl;
use std::time::Duration;

use serde::{Serialize, de::DeserializeOwned};

use crate::error::Result;
#[cfg(feature = "moka")]
pub use moka::MokaCacheImpl as Cache;

#[cfg(feature = "redis")]
pub use redis::RedisCacheImpl as Cache;

pub struct IterItem {
    pub key: String,
    pub expired_at: Option<u64>,
}
pub struct JsonItem {
    pub value: serde_json::Value,
    pub expired_at: Option<u64>,
}
pub trait CacheTrait: Clone {
    fn get<T>(&self, key: &str) -> impl Future<Output = Option<T>>
    where
        T: DeserializeOwned;
    fn insert<T>(&self, key: &str, value: T) -> impl Future<Output = Result<()>>
    where
        T: Serialize;
    fn insert_with_ttl<T>(
        &self,
        key: &str,
        value: T,
        ttl: Duration,
    ) -> impl Future<Output = Result<()>>
    where
        T: Serialize;
    fn iter_prefix(&self, prefix: &str) -> impl Future<Output = Result<Vec<IterItem>>>;
    fn delete_prefix(&self, prefix: &str) -> impl Future<Output = Result<()>>;
    fn get_raw_string(&self, key: &str) -> impl Future<Output = Option<JsonItem>>;
}
