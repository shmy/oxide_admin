use infrastructure::shared::cloneable_error::CloneableError;
use moka::future::Cache;
use std::hash::{Hash, Hasher as _};
use twox_hash::XxHash64;

pub type CacheType<T> = Cache<u64, Result<T, CloneableError>>;

#[macro_export]
macro_rules! impl_static_cache {
    ($name: ident, $ty: ty, $capacity: expr, $time_to_live: expr) => {
        static $name: std::sync::LazyLock<CacheType<$ty>> = std::sync::LazyLock::new(|| {
            moka::future::Cache::builder()
                .max_capacity($capacity)
                .time_to_live($time_to_live)
                .build()
        });
    };
}

pub fn hash_encode(query: &impl Hash) -> u64 {
    let mut hasher = XxHash64::default();
    query.hash(&mut hasher);
    hasher.finish()
}
