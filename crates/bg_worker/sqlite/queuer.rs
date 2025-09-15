use std::fmt::Debug;

use anyhow::Result;
use serde::Serialize;
use sqlx::FromRow;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone)]
pub struct Queuer {
    pool: sqlx::SqlitePool,
    sender: Sender<i64>,
}

impl Debug for Queuer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queuer").finish()
    }
}

impl Queuer {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        let (sender, _) = broadcast::channel(16);

        let instance = Self { pool, sender };
        instance.resume();
        instance
    }

    pub(crate) fn subscribe(&self) -> Receiver<i64> {
        self.sender.subscribe()
    }

    pub(crate) fn pool(&self) -> sqlx::SqlitePool {
        self.pool.clone()
    }

    pub(crate) fn resume(&self) {
        let pool = self.pool.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let pending_job_rows: Vec<PendingJobRow> =
                sqlx::query_as("SELECT rowid as rowid FROM _jobs WHERE status = 'pending'")
                    .fetch_all(&pool)
                    .await?;
            for ele in pending_job_rows {
                let id = ele.rowid;
                sender.send(id)?;
            }
            anyhow::Ok(())
        });
    }

    pub(crate) fn delete_outdated(&self) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            sqlx::query("DELETE FROM _jobs WHERE status = 'done'")
                .execute(&pool)
                .await?;
            anyhow::Ok(())
        });
    }

    pub async fn enqueue<K, V>(&self, kind: K, args: V) -> Result<()>
    where
        K: Into<String>,
        V: Serialize,
    {
        let id =sqlx::query(r#"
        INSERT INTO _jobs (kind, args, status, created_at, updated_at) VALUES (?, ?, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#)
            .bind(kind.into())
            .bind(serde_json::to_string(&args)?)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        self.sender.send(id)?;
        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct PendingJobRow {
    rowid: i64,
}
