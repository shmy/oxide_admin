use anyhow::Result;
use faktory::{Job, WorkerBuilder};

use crate::{JobRunner, RunnerWrapper, error::RunnerError};

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

    pub fn register_fn<K, H, Fut>(&mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), RunnerError>> + Send,
    {
        let old = std::mem::take(&mut self.worker_builder);
        self.worker_builder = old.register_fn(kind, handler);
    }

    pub fn register_blocking_fn<K, H>(mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(Job) -> Result<(), RunnerError> + Send + Sync + 'static,
    {
        let old = std::mem::take(&mut self.worker_builder);
        self.worker_builder = old.register_blocking_fn(kind, handler);
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
