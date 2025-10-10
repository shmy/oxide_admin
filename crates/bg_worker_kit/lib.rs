pub mod error;
use std::time::Duration;

use crate::error::{Result, WorkerError};
use apalis::layers::WorkerBuilderExt as _;
use apalis::layers::retry::RetryPolicy;
pub use apalis::prelude::BackendExpose;
pub use apalis::prelude::Stat;
use apalis::prelude::{Data, Monitor, WorkerBuilder, WorkerFactoryFn as _};
use apalis_core::codec::json::JsonCodec;
pub use apalis_core::storage::Storage;
use serde::{Serialize, de::DeserializeOwned};
#[cfg(feature = "sqlite")]
pub type Pool = apalis_sql::sqlite::SqlitePool;
#[cfg(feature = "postgres")]
pub type Pool = apalis_sql::postgres::PgPool;
#[cfg(feature = "sqlite")]
pub type StorageBackend<T, C = JsonCodec<String>> = apalis_sql::sqlite::SqliteStorage<T, C>;
#[cfg(feature = "postgres")]
pub type StorageBackend<T, C = JsonCodec<serde_json::Value>> =
    apalis_sql::postgres::PostgresStorage<T, C>;

#[cfg(feature = "sqlite")]
pub mod sqlite_helper;

pub struct WorkerManager {
    pool: Pool,
    monitor: Monitor,
    table_name: &'static str,
}

impl WorkerManager {
    pub async fn try_new(pool: Pool) -> Result<Self> {
        #[cfg(feature = "sqlite")]
        let table_name = "Jobs";
        #[cfg(feature = "postgres")]
        let table_name = "apalis.jobs";
        let instance = Self {
            pool: pool.clone(),
            monitor: Monitor::new(),
            table_name,
        };
        instance.setup().await?;
        Ok(instance)
    }
}

impl WorkerManager {
    async fn setup(&self) -> Result<()> {
        #[cfg(feature = "sqlite")]
        tracing::info!("Choosing database backend: SQLite");
        #[cfg(feature = "postgres")]
        tracing::info!("Choosing database backend: PosgreSQL");
        StorageBackend::setup(&self.pool).await?;
        let mut tx = self.pool.acquire().await?;
        let query = format!("DELETE FROM {} WHERE status='Done'", self.table_name);
        sqlx::query(&query).execute(&mut *tx).await?;
        let query = format!(
            "UPDATE {} SET status = 'Pending', done_at = NULL, lock_by = NULL, lock_at = NULL, last_error = 'Job was abandoned' WHERE status = 'Running'",
            self.table_name
        );
        sqlx::query(&query).execute(&mut *tx).await?;
        Ok(())
    }

    pub fn register<T>(mut self, data: T::State) -> (Self, StorageBackend<T>)
    where
        T: Worker,
    {
        let backend = StorageBackend::new(self.pool.clone());
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
        (self, backend)
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
