pub mod error;
use std::time::Duration;

use crate::error::{Result, WorkerError};
use apalis::layers::WorkerBuilderExt as _;
use apalis::layers::retry::RetryPolicy;
use apalis::prelude::{Data, Monitor, WorkerBuilder, WorkerFactoryFn as _};
use apalis_core::codec::json::JsonCodec;
pub use apalis_core::storage::Storage;
use serde::{Serialize, de::DeserializeOwned};
pub type Pool = apalis_sql::postgres::PgPool;
pub type StorageBackend<T, C = JsonCodec<serde_json::Value>> =
    apalis_sql::postgres::PostgresStorage<T, C>;

pub struct WorkerManager {
    pool: Pool,
    monitor: Monitor,
}

impl WorkerManager {
    pub async fn try_new(pool: Pool) -> Result<Self> {
        let instance = Self {
            pool,
            monitor: Monitor::new(),
        };
        instance.migrate().await?;
        instance.resume().await?;
        Ok(instance)
    }
}

impl WorkerManager {
    async fn migrate(&self) -> Result<()> {
        StorageBackend::setup(&self.pool).await?;
        Ok(())
    }

    async fn resume(&self) -> Result<()> {
        let query = "Update apalis.jobs SET status = 'Pending', done_at = NULL, lock_by = NULL, lock_at = NULL, last_error = 'Job was abandoned' WHERE status = 'Running'";
        let mut tx = self.pool.acquire().await?;

        sqlx::query(query).execute(&mut *tx).await?;

        Ok(())
    }

    pub fn register<T>(mut self, backend: StorageBackend<T>, data: T::State) -> Self
    where
        T: Worker,
    {
        let worker = WorkerBuilder::new(T::NAME)
            .enable_tracing()
            .concurrency(T::CONCURRENCY)
            .retry(RetryPolicy::retries(T::RETRIES))
            .data(data)
            .backend(backend.clone())
            .build_fn(|job: T, data: Data<T::State>| async move {
                match tokio::time::timeout(T::TIMEOUT, T::execute(job, &data)).await {
                    Ok(res) => res,
                    Err(_) => Err(WorkerError::Timeout),
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
        self.pool.close().await;
        Ok(())
    }
}

pub trait Worker: Serialize + DeserializeOwned + Clone + Send + Sync + Unpin + 'static {
    type State: Clone + Send + Sync + Unpin + 'static;
    const NAME: &'static str;
    const CONCURRENCY: usize;
    const RETRIES: usize;
    const TIMEOUT: Duration;

    fn execute(job: Self, state: &Self::State) -> impl Future<Output = Result<()>> + Send;
}
