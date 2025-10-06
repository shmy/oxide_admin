use std::time::Duration;

use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};
pub mod error;

#[cfg(feature = "redb")]
mod redb;
#[cfg(feature = "redis")]
mod redis;
mod serde_util;

#[cfg(feature = "redb")]
pub type Kvdb = crate::redb::RedbKvdb;
#[cfg(feature = "redis")]
pub type Kvdb = crate::redis::RedisKvdb;
#[cfg(feature = "redis")]
pub use redis::RedisKvdbConfig;

pub struct IterItem {
    pub key: String,
    pub expired_at: Option<i64>,
}

pub trait KvdbTrait: Clone {
    fn get<T: DeserializeOwned>(&self, key: &str) -> impl Future<Output = Option<T>>;
    fn set_with_ex<T: Serialize>(
        &self,
        key: &str,
        value: T,
        duration: Duration,
    ) -> impl Future<Output = Result<()>>;
    fn set_with_ex_at<T: Serialize>(
        &self,
        key: &str,
        value: T,
        expires_at: i64,
    ) -> impl Future<Output = Result<()>>;
    fn set<T: Serialize>(&self, key: &str, value: T) -> impl Future<Output = Result<()>>;
    fn delete(&self, key: &str) -> impl Future<Output = Result<()>>;
    fn delete_prefix(&self, prefix: &str) -> impl Future<Output = Result<()>>;
    fn iter_prefix(&self, prefix: &str) -> impl Future<Output = Result<Vec<IterItem>>>;
    fn close(&self) -> impl Future<Output = ()>;
}
