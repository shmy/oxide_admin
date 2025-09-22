use std::{collections::HashMap, sync::Arc};

use crate::{Worker, error::RunnerError, queuer::Queuer};
use anyhow::Result;
use futures_util::{FutureExt, future::BoxFuture};
use sqlx::FromRow;
use tracing::{error, info};

struct RunnerWrapper<T>(pub T)
where
    T: Worker + Clone + Send + Sync + 'static;
trait WorkerDyn {
    fn run(
        &self,
        row: JobRow,
        pool: sqlx::SqlitePool,
    ) -> BoxFuture<'static, Result<(), RunnerError>>;
}

impl<T> WorkerDyn for RunnerWrapper<T>
where
    T: Worker + Clone + Send + Sync + 'static,
{
    fn run(
        &self,
        row: JobRow,
        pool: sqlx::SqlitePool,
    ) -> BoxFuture<'static, Result<(), RunnerError>> {
        let inner = self.0.clone();
        async move {
            let params: T::Params = serde_json::from_str(&row.params)?;
            match inner.run(params).await {
                Ok(_) => {
                    info!("Job {} run with {} successfully", row.kind, &row.params);
                    sqlx::query("DELETE FROM _jobs WHERE id = ?")
                        .bind(row.id)
                        .execute(&pool)
                        .await
                        .map_err(|e| RunnerError::Custom(e.to_string()))?;
                },
                Err(err) => {
                    error!("Job {} run with {} failed: {}", row.kind, &row.params, err);
                    sqlx::query("UPDATE _jobs SET status = 'error', reason = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(err.to_string())
                        .bind(row.id)
                        .execute(&pool)
                        .await
                        .map_err(|e| RunnerError::Custom(e.to_string()))?;
                },
            }
            Ok(())
        }
        .boxed()
    }
}

pub struct WorkerManager {
    queuer: Queuer,
    runners: HashMap<String, Arc<dyn WorkerDyn + Send + Sync + 'static>>,
}

impl WorkerManager {
    pub fn new(queuer: Queuer) -> Self {
        Self {
            queuer,
            runners: HashMap::new(),
        }
    }

    pub fn register<K, R>(&mut self, kind: K, runner: R)
    where
        K: Into<String>,
        R: Worker + Clone + Send + Sync + 'static,
    {
        let kind = kind.into();
        self.runners.insert(kind, Arc::new(RunnerWrapper(runner)));
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        tokio::pin!(signal);
        let mut receiver = self.queuer.subscribe();
        self.queuer.resume();
        self.queuer.delete_outdated();
        loop {
            tokio::select! {
                maybe_id = receiver.recv() => {
                    let pool = self.queuer.pool();
                    if let Ok(id) = maybe_id {
                        info!("Job received rowid: {}", id);
                        let job_row: JobRow = sqlx::query_as("SELECT id, kind, params FROM _jobs WHERE rowid = ?").bind(id)
                            .fetch_one(&pool)
                            .await?;
                         if let Some(runner) = self.runners.get(&job_row.kind) {
                            info!("Job handle found: {}", job_row.kind);
                            tokio::spawn(runner.run(job_row, pool));
                        }
                    } else {
                        break;
                    }
                }
                _ = &mut signal => {
                    break;
                }
            }
        }

        self.queuer.pool().close().await;
        Ok(())
    }
}

#[derive(FromRow)]
struct JobRow {
    id: i64,
    kind: String,
    params: String,
}
