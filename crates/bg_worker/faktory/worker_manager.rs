use std::sync::Arc;

use crate::{JobRunner, RunnerWrapper, error::RunnerError};
use anyhow::Result;
use faktory::WorkerBuilder;
use serde::{Serialize, de::DeserializeOwned};

pub struct WorkerManager {
    addr: String,
    queue: String,
    worker_builder: WorkerBuilder<RunnerError>,
}

impl WorkerManager {
    pub fn new(addr: impl Into<String>, queue: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            queue: queue.into(),
            worker_builder: WorkerBuilder::default(),
        }
    }

    pub fn register<K, P, H>(&mut self, kind: K, runner: H)
    where
        K: Into<String>,
        H: JobRunner<Params = P> + Send + Sync + 'static,
    {
        let old = std::mem::take(&mut self.worker_builder);
        self.worker_builder = old.register(kind, RunnerWrapper(runner));
    }

    pub fn register_fn<K, H, P, Fut>(&mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Fut + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
        Fut: Future<Output = Result<(), RunnerError>> + Send,
    {
        let old = std::mem::take(&mut self.worker_builder);
        let handler = Arc::new(handler);
        self.worker_builder = old.register_fn(kind, move |job| {
            let handler = handler.clone();
            async move {
                if let Some(arg) = job.args().first() {
                    let params: P = serde_json::from_value(arg.clone())?;
                    return handler(params).await;
                }
                Err(RunnerError::Custom("No params".to_string()))
            }
        });
    }

    pub fn register_blocking_fn<K, H, P>(mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Result<(), RunnerError> + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
    {
        let old = std::mem::take(&mut self.worker_builder);
        self.worker_builder = old.register_blocking_fn(kind, move |job| {
            if let Some(arg) = job.args().first() {
                let params: P = serde_json::from_value(arg.clone())?;
                return handler(params);
            }
            Err(RunnerError::Custom("No params".to_string()))
        });
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        let old = std::mem::take(&mut self.worker_builder);
        let mut worker = old
            .with_graceful_shutdown(signal)
            .connect_to(&self.addr)
            .await?;
        worker.run(&[&self.queue]).await?;
        Ok(())
    }
}
