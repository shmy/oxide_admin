use moka::future::Cache;
use std::hash::{Hash, Hasher as _};
use twox_hash::XxHash64;

pub type CacheType<T, E> = Cache<u64, Result<T, E>>;

#[macro_export]
macro_rules! impl_static_cache {
    ($name: ident, $ty: ty, $err: ty, $capacity: expr, $time_to_live: expr) => {
        static $name: std::sync::LazyLock<CacheType<$ty, $err>> = std::sync::LazyLock::new(|| {
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
