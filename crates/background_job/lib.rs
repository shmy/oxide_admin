use anyhow::Result;
pub use apalis::prelude::{Data, Error, GoTo, StepBuilder, SteppableStorage, Storage};
use apalis::{
    layers::{WorkerBuilderExt as _, retry::RetryPolicy},
    prelude::*,
};
pub use apalis_core::codec::json::JsonCodec;
pub use apalis_core::step::StepFn;
pub use apalis_sql::context::SqlContext;
use apalis_sql::{sqlite::SqliteStorage, sqlx};
use serde::{Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};
pub type JobStorage<T> = SqliteStorage<T>;
pub type SteppedJobStorage = SqliteStorage<StepRequest<String>>;

pub struct BackgroundJobManager {
    pool: sqlx::SqlitePool,
    monitor: Monitor,
}

impl BackgroundJobManager {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            pool,
            monitor: Monitor::new(),
        }
    }

    pub async fn migrate(self) -> Result<Self> {
        SqliteStorage::setup(&self.pool).await?;
        Ok(self)
    }

    pub async fn resume(&self) -> Result<()> {
        let query = r#"Update Jobs SET status = 'Pending', done_at = NULL, lock_by = NULL, lock_at = NULL, last_error = NULL WHERE status = 'Running'"#;
        sqlx::query(query).execute(&self.pool).await?;

        Ok(())
    }

    pub fn register<T>(mut self, job: T, storage: SqliteStorage<T::Params>) -> Self
    where
        T: Job,
    {
        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .concurrency(T::CONCURRENCY)
            .retry(RetryPolicy::retries(T::RETRIES))
            .backend(storage.clone())
            .build_fn(move |params: T::Params| {
                let job = job.clone();
                async move {
                    match tokio::time::timeout(T::TIMEOUT, job.execute(params)).await {
                        Ok(res) => res.map_err(|err| Error::Failed(Arc::new(err.into()))),
                        Err(e) => Err(Error::Abort(Arc::new(e.into()))),
                    }
                }
            });
        self.monitor = self.monitor.register(worker);
        self
    }

    pub fn register_stepped<T>(
        mut self,
        data: T::State,
    ) -> (Self, SqliteStorage<StepRequest<String>>)
    where
        T: SteppedJob,
    {
        let storage = SqliteStorage::new(self.pool.clone());
        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .concurrency(T::CONCURRENCY)
            .data(data)
            .backend(storage.clone())
            .build_stepped(T::steps());
        self.monitor = self.monitor.register(worker);
        (self, storage)
    }

    pub async fn run_with_signal<S>(self, signal: S) -> Result<()>
    where
        S: Send + Future<Output = std::io::Result<()>>,
    {
        self.monitor.run_with_signal(signal).await?;
        self.pool.close().await;
        Ok(())
    }
}
pub trait JobParams: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static {}

impl<T> JobParams for T where T: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static
{}

pub trait Job: Clone + Send + Sync + Unpin + 'static {
    type Params: JobParams;
    const NAME: &'static str;
    const CONCURRENCY: usize;
    const RETRIES: usize;
    const TIMEOUT: Duration;

    fn execute(&self, params: Self::Params) -> impl Future<Output = Result<()>> + Send;
}

pub trait SteppedJob: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static {
    type State: Clone + Send + Sync + Unpin + 'static;
    type Input: Clone + Send + Sync + Unpin + 'static;
    const NAME: &'static str;
    const CONCURRENCY: usize;

    fn steps() -> StepBuilder<SqlContext, String, Self::Input, (), JsonCodec<String>, usize>;
}
