use anyhow::Result;
use serde::Serialize;

#[derive(Clone)]
pub struct Queuer {
    pool: sqlx::SqlitePool,
}

impl Queuer {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> sqlx::SqlitePool {
        self.pool.clone()
    }

    pub async fn enqueue<K, V>(&self, kind: K, args: V) -> Result<()>
    where
        K: Into<String>,
        V: Serialize,
    {
        sqlx::query(r#""INSERT INTO queues (kind, args) VALUES (?1, ?2)"#)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
