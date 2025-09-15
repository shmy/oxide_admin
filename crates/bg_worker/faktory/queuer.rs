use anyhow::Result;
use faktory::{Client, Job};
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Queuer {
    client: Arc<Mutex<Client>>,
    queue: String,
}

impl Debug for Queuer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queuer").finish()
    }
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
        V: Serialize,
    {
        let mut guard = self.client.lock().await;
        guard
            .enqueue(
                Job::builder(kind.into())
                    .queue(&self.queue)
                    .args(vec![serde_json::to_value(args)?])
                    .build(),
            )
            .await?;
        Ok(())
    }
}
