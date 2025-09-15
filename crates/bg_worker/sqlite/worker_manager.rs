use std::{collections::HashMap, sync::Arc};

use crate::{JobRunner, error::RunnerError};
use anyhow::Result;
use futures_util::{FutureExt, future::BoxFuture};
use sqlx::FromRow;
use tokio::sync::broadcast::Receiver;

struct RunnerWrapper<T>(pub T)
where
    T: JobRunner + Clone + Send + Sync + 'static;
trait JobRunnerDyn {
    fn run(
        &self,
        row: JobRow,
        pool: sqlx::SqlitePool,
    ) -> BoxFuture<'static, Result<(), RunnerError>>;
}

impl<T> JobRunnerDyn for RunnerWrapper<T>
where
    T: JobRunner + Clone + Send + Sync + 'static,
{
    fn run(
        &self,
        row: JobRow,
        pool: sqlx::SqlitePool,
    ) -> BoxFuture<'static, Result<(), RunnerError>> {
        let inner = self.0.clone();
        async move {
            let params: T::Params = serde_json::from_str(&row.args)?;
            let (status, reason) = match inner.run(params).await {
                Ok(_) => ("done".to_string(), None),
                Err(err) => ("error".to_string(), Some(err.to_string())),
            };
            sqlx::query("UPDATE _jobs SET status = ?1, reason = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3")
                .bind(status)
                .bind(reason)
                .bind(row.id)
                .execute(&pool)
                .await
                .map_err(|e| RunnerError::Custom(e.to_string()))?;
            Ok(())
        }
        .boxed()
    }
}

pub struct WorkerManager {
    pool: sqlx::SqlitePool,
    receiver: Receiver<i64>,
    runners: HashMap<String, Arc<dyn JobRunnerDyn + Send + Sync + 'static>>,
}

impl WorkerManager {
    pub fn new(pool: sqlx::SqlitePool, receiver: Receiver<i64>) -> Self {
        Self {
            pool,
            receiver,
            runners: HashMap::new(),
        }
    }

    pub fn register<K, R>(&mut self, kind: K, runner: R)
    where
        K: Into<String>,
        R: JobRunner + Clone + Send + Sync + 'static,
    {
        let kind = kind.into();
        self.runners.insert(kind, Arc::new(RunnerWrapper(runner)));
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        tokio::pin!(signal);
        loop {
            tokio::select! {
                maybe_id = self.receiver.recv() => {
                    let pool = self.pool.clone();
                    if let Ok(id) = maybe_id {
                        let job_row: JobRow = sqlx::query_as("SELECT id, kind, args FROM _jobs WHERE rowid = ?").bind(id)
                            .fetch_one(&pool)
                            .await?;
                         if let Some(runner) = self.runners.get(&job_row.kind) {
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

        self.pool.close().await;
        Ok(())
    }
}

#[derive(FromRow)]
struct JobRow {
    id: i64,
    kind: String,
    args: String,
}
