use anyhow::Result;
pub use apalis::prelude::{Data, Error, GoTo, StepBuilder, SteppableStorage, Storage};
use apalis::{
    layers::{WorkerBuilderExt as _, retry::RetryPolicy},
    prelude::*,
};
pub use apalis_core::codec::json::JsonCodec;
pub use apalis_core::step::StepFn;
use apalis_cron::{CronContext, CronStream, Schedule};
pub use apalis_sql::context::SqlContext;
use apalis_sql::{
    sqlite::SqliteStorage,
    sqlx::{self, types::chrono::Local},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{str::FromStr as _, sync::Arc, time::Duration};
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
            .backend(storage)
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

    pub fn register_stepped<T>(mut self, storage: SqliteStorage<StepRequest<String>>) -> Self
    where
        T: SteppedJob,
    {
        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .concurrency(T::CONCURRENCY)
            .backend(storage)
            .build_stepped(T::steps());
        self.monitor = self.monitor.register(worker);
        self
    }

    pub fn register_cron<T>(mut self, job: T) -> Self
    where
        T: CronJob,
    {
        let cron = english_to_cron::str_cron_syntax(T::SCHEDULE).expect("build cron syntax");
        let schedule = Schedule::from_str(&cron).expect("build schedule");
        let cron_stream = CronStream::new(schedule);
        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .backend(cron_stream)
            .build_fn(move |_ctx: CronContext<Local>| {
                let job = job.clone();
                async move {
                    if let Err(err) = tokio::time::timeout(T::TIMEOUT, job.execute()).await {
                        tracing::error!(%err, "Failed to run cron job {}", T::NAME);
                    }
                }
            });
        self.monitor = self.monitor.register(worker);
        self
    }

    pub async fn run_with_signal<S>(self, signal: S) -> Result<()>
    where
        S: Send + Future<Output = std::io::Result<()>>,
    {
        self.monitor.run_with_signal(signal).await?;
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

pub trait SteppedJob: Clone + Send + Sync + Unpin + 'static {
    type Params: JobParams;
    const NAME: &'static str;
    const CONCURRENCY: usize;

    fn steps() -> StepBuilder<SqlContext, String, Self::Params, (), JsonCodec<String>, usize>;
}

pub trait CronJob: Clone + Send + Sync + Unpin + 'static {
    const NAME: &'static str;
    const SCHEDULE: &'static str;
    const TIMEOUT: Duration;
    fn execute(&self) -> impl Future<Output = Result<()>> + Send;
}
