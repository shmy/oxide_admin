use std::sync::Arc;

use anyhow::Result;
use faktory::{Client, Job};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Publisher {
    client: Arc<Mutex<Client>>,
    queue: String,
}

impl Publisher {
    pub async fn try_new(addr: impl Into<String>, queue: impl Into<String>) -> Result<Self> {
        let client = Client::connect_to(&addr.into()).await?;
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            queue: queue.into(),
        })
    }

    pub async fn publish<K, V>(&self, kind: K, args: V) -> Result<()>
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
