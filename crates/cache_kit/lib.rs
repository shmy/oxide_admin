mod error;
#[cfg(feature = "moka")]
pub mod moka;
mod serde_util;
use std::time::Duration;

use serde::{Serialize, de::DeserializeOwned};

use crate::error::Result;

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
}
