use std::{
    hash::{Hash, Hasher as _},
    time::Duration,
};

use anyhow::Result;
use bon::Builder;
use infrastructure::shared::kv::{Kv, KvTrait as _};
use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;
use twox_hash::XxHash64;

fn hash_encode(query: &impl Hash) -> u64 {
    let mut hasher = XxHash64::default();
    query.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Builder)]
pub struct CacheProvider {
    prefix: &'static str,
    ttl: Duration,
    kv: Kv,
}

impl CacheProvider {
    pub async fn clear(&self) -> Result<()> {
        self.kv.delete_prefix(self.prefix).await?;
        Ok(())
    }

    pub async fn get_with<K, V, E, F, Fut>(&self, key: K, resolve: F) -> Result<V, E>
    where
        K: Clone + Hash + Eq,
        V: Clone + Serialize + DeserializeOwned,
        F: FnOnce(K) -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        let cache_key = format!("{}{}", self.prefix, hash_encode(&key));
        if let Some(cache) = self.kv.get::<V>(&cache_key).await {
            return Ok(cache);
        }
        let value = resolve(key).await?;
        if let Err(err) = self
            .kv
            .set_with_ex(&cache_key, value.clone(), self.ttl)
            .await
        {
            warn!(%err, "failed to set cache {}", &cache_key)
        }
        Ok(value)
    }
}
