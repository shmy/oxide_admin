use anyhow::Result;
use serde::Serialize;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct Queuer {}
impl Queuer {
    pub async fn try_new() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn enqueue<K, V>(&self, _kind: K, _params: V) -> Result<()>
    where
        K: Into<String>,
        V: Serialize,
    {
        warn!("Dummy queuer is used, this is not a real queuer");
        Ok(())
    }
}
