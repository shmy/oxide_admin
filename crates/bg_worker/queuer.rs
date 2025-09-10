#[cfg(feature = "faktory")]
#[derive(Clone)]
pub struct Queuer {
    client: std::sync::Arc<tokio::sync::Mutex<faktory::Client>>,
    queue: String,
}
#[cfg(feature = "faktory")]
impl Queuer {
    pub async fn try_new(
        addr: impl Into<String>,
        queue: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let client = faktory::Client::connect_to(&addr.into()).await?;
        Ok(Self {
            client: std::sync::Arc::new(tokio::sync::Mutex::new(client)),
            queue: queue.into(),
        })
    }

    pub async fn enqueue<K, V>(&self, kind: K, args: V) -> anyhow::Result<()>
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        let mut guard = self.client.lock().await;
        guard
            .enqueue(
                faktory::Job::builder(kind.into())
                    .queue(&self.queue)
                    .args(vec![args])
                    .build(),
            )
            .await?;
        Ok(())
    }
}

#[cfg(not(feature = "faktory"))]
#[derive(Clone)]
pub struct Queuer {}
#[cfg(not(feature = "faktory"))]
impl Queuer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn enqueue<K, V>(&self, _kind: K, _args: V) -> anyhow::Result<()>
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        Ok(())
    }
}
