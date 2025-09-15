use crate::{JobRunner, error::RunnerError};
use anyhow::Result;
use faktory::WorkerBuilder;

struct RunnerWrapper<T>(pub T)
where
    T: JobRunner;

#[async_trait::async_trait]
impl<T> ::faktory::JobRunner for RunnerWrapper<T>
where
    T: JobRunner + Send + Sync + 'static,
{
    type Error = RunnerError;
    async fn run(&self, job: ::faktory::Job) -> Result<(), Self::Error> {
        if let Some(arg) = job.args().first() {
            let params: T::Params = serde_json::from_value(arg.clone())?;
            return self.0.run(params).await;
        }
        Err(RunnerError::Custom("No params".to_string()))
    }
}

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
