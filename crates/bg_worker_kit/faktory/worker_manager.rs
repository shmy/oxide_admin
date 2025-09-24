use crate::error::Result;
use crate::{Worker, error::WorkerError};
use faktory::WorkerBuilder;

struct RunnerWrapper<T>(pub T)
where
    T: Worker;

#[async_trait::async_trait]
impl<T> ::faktory::JobRunner for RunnerWrapper<T>
where
    T: Worker + Send + Sync + 'static,
{
    type Error = WorkerError;
    async fn run(&self, job: ::faktory::Job) -> Result<()> {
        if let Some(arg) = job.args().first() {
            let params: T::Params = serde_json::from_value(arg.clone())?;
            return self.0.run(params).await;
        }
        Err(WorkerError::Custom("No params".to_string()))
    }
}

pub struct WorkerManager {
    addr: String,
    queue: String,
    worker_builder: WorkerBuilder<WorkerError>,
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
        H: Worker<Params = P> + Send + Sync + 'static,
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
