use std::time::Duration;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

use crate::port::redb_kv_impl::RedbKv;

pub type Kv = RedbKv;

pub trait KvTrait {
    fn get<T: DeserializeOwned + Default>(&self, key: &str) -> impl Future<Output = Result<T>>;
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
    fn delete_expired(&self) -> impl Future<Output = Result<()>>;
}
