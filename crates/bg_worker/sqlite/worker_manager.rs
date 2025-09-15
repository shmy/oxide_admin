use crate::{JobRunner, error::RunnerError};
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

pub struct WorkerManager {
    pool: sqlx::SqlitePool,
}

impl WorkerManager {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub fn register<K, P, H>(&mut self, kind: K, runner: H)
    where
        K: Into<String>,
        H: JobRunner<Params = P> + Send + Sync + 'static,
    {
    }

    pub fn register_fn<K, H, P, Fut>(&mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Fut + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
        Fut: Future<Output = Result<(), RunnerError>> + Send,
    {
    }

    pub fn register_blocking_fn<K, H, P>(mut self, kind: K, handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Result<(), RunnerError> + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
    {
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        let mut conn = self.pool.acquire().await?;
        let mut handle = conn.lock_handle().await?;
        handle.set_update_hook(|s| {
            println!("update hook: {:?}", s.database);
            println!("update hook: {:?}", s.table);
            println!("update hook: {:?}", s.operation);
            println!("update hook: {:?}", s.rowid);
        });
        signal.await;
        drop(handle);
        drop(conn);
        self.pool.close().await;
        Ok(())
    }
}
