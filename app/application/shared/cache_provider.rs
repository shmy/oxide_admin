use std::{
    hash::{Hash, Hasher as _},
    time::Duration,
};

use crate::error::ApplicationResult;
use bon::Builder;
use kvdb_kit::{Kvdb, KvdbTrait as _};
use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;
use twox_hash::XxHash64;

fn hash_encode(query: &impl Hash) -> u64 {
    let mut hasher = XxHash64::default();
    query.hash(&mut hasher);
    hasher.finish()
}

pub const CACHE_PREFIX: &str = "CACHE:";
#[derive(Debug, Clone, Builder)]
pub struct CacheProvider {
    key: &'static str,
    ttl: Duration,
    kvdb: Kvdb,
}

impl CacheProvider {
    fn fill_key(&self) -> String {
        format!("{}{}", CACHE_PREFIX, self.key)
    }
    #[tracing::instrument]
    pub async fn clear(&self) -> ApplicationResult<()> {
        self.kvdb.delete_prefix(&self.fill_key()).await?;
        Ok(())
    }

    #[tracing::instrument(skip(key, resolve))]
    pub async fn get_with<K, V, E, F, Fut>(&self, key: K, resolve: F) -> Result<V, E>
    where
        K: Clone + Hash + Eq,
        V: Clone + Serialize + DeserializeOwned,
        F: FnOnce(K) -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        let cache_key = format!("{}{}", &self.fill_key(), hash_encode(&key));
        if let Some(cache) = self.kvdb.get::<V>(&cache_key).await {
            return Ok(cache);
        }
        let value = resolve(key).await?;
        if let Err(err) = self
            .kvdb
            .set_with_ex(&cache_key, value.clone(), self.ttl)
            .await
        {
            warn!(%err, "failed to set cache {}", &cache_key)
        }
        Ok(value)
    }
}
