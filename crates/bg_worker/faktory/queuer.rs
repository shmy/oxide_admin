use std::sync::Arc;

use anyhow::Result;
use faktory::{Client, Job};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Queuer {
    client: Arc<Mutex<Client>>,
    queue: String,
}
impl Queuer {
    pub async fn try_new(addr: impl Into<String>, queue: impl Into<String>) -> Result<Self> {
        let mut client = Client::connect_to(&addr.into()).await?;
        let info = client.current_info().await?;
        tracing::info!("Faktory version: {} connected", info.server.version);
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            queue: queue.into(),
        })
    }

    pub async fn enqueue<K, V>(&self, kind: K, args: V) -> Result<()>
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        let mut guard = self.client.lock().await;
        guard
            .enqueue(
                Job::builder(kind.into())
                    .queue(&self.queue)
                    .args(vec![args])
                    .build(),
            )
            .await?;
        Ok(())
    }
}
