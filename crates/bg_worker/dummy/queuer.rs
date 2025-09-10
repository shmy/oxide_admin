use anyhow::Result;
use tracing::warn;

#[derive(Clone)]
pub struct Queuer {}
impl Queuer {
    pub async fn try_new() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn enqueue<K, V>(&self, _kind: K, _args: V) -> Result<()>
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        warn!("Dummy queuer is used, this is not a real queuer");
        Ok(())
    }
}
