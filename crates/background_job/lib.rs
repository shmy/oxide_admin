use anyhow::Result;
pub use apalis::prelude::{Data, Error, GoTo, StepBuilder, SteppableStorage, Storage};
use apalis::{
    layers::{WorkerBuilderExt as _, retry::RetryPolicy},
    prelude::*,
};
pub use apalis_core::codec::json::JsonCodec;
pub use apalis_core::step::StepFn;
pub use apalis_sql::context::SqlContext;
use apalis_sql::sqlx::ConnectOptions as _;
use apalis_sql::{
    sqlite::SqliteStorage,
    sqlx::{
        self,
        sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    },
};
use serde::{Serialize, de::DeserializeOwned};
use std::{path::Path, sync::Arc, time::Duration};
use tracing::log::LevelFilter;
pub type JobStorage<T> = SqliteStorage<T>;
pub type SteppedJobStorage = SqliteStorage<StepRequest<String>>;

pub struct BackgroundJobManager {
    pool: sqlx::SqlitePool,
    monitor: Monitor,
}

impl BackgroundJobManager {
    pub async fn try_new(path: impl AsRef<Path>) -> Result<Self> {
        let connection_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);
        let connection_options = connection_options.log_statements(LevelFilter::Trace);

        let pool = SqlitePoolOptions::default()
            .connect_with(connection_options)
            .await?;
        Ok(Self {
            pool,
            monitor: Monitor::new(),
        })
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

    pub fn register<T>(mut self, data: T::State) -> (Self, SqliteStorage<T>)
    where
        T: Job,
    {
        let storage: SqliteStorage<T> = SqliteStorage::new(self.pool.clone());

        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .concurrency(T::CONCURRENCY)
            .retry(RetryPolicy::retries(T::RETRIES))
            .data(data)
            .backend(storage.clone())
            .build_fn(|job: T, data: Data<T::State>| async move {
                match tokio::time::timeout(T::TIMEOUT, T::execute(job, &data)).await {
                    Ok(res) => res.map_err(|err| Error::Failed(Arc::new(err.into()))),
                    Err(e) => Err(Error::Abort(Arc::new(e.into()))),
                }
            });
        self.monitor = self.monitor.register(worker);
        (self, storage)
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

pub trait Job: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static {
    type State: Clone + Send + Sync + Unpin + 'static;
    const NAME: &'static str;
    const CONCURRENCY: usize;
    const RETRIES: usize;
    const TIMEOUT: Duration;

    fn execute(job: Self, state: &Self::State) -> impl Future<Output = Result<()>> + Send;
}

pub trait SteppedJob: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static {
    type State: Clone + Send + Sync + Unpin + 'static;
    type Input: Clone + Send + Sync + Unpin + 'static;
    const NAME: &'static str;
    const CONCURRENCY: usize;

    fn steps() -> StepBuilder<SqlContext, String, Self::Input, (), JsonCodec<String>, usize>;
}
