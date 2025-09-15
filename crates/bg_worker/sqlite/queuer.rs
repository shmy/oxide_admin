use std::fmt::Debug;

use anyhow::Result;
use serde::Serialize;
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
        Self { pool, sender }
    }

    pub fn subscribe(&self) -> Receiver<i64> {
        self.sender.subscribe()
    }

    pub fn pool(&self) -> sqlx::SqlitePool {
        self.pool.clone()
    }

    pub async fn enqueue<K, V>(&self, kind: K, args: V) -> Result<()>
    where
        K: Into<String>,
        V: Serialize,
    {
        let id =sqlx::query(r#"
        INSERT INTO _jobs (kind, args, status, created_at, updated_at) VALUES (?1, ?2, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
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
