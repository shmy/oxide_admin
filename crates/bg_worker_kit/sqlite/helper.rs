use std::path::Path;

use crate::error::Result;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tracing::log::LevelFilter;

use crate::sqlite::migration;

pub async fn connect_sqlite(path: impl AsRef<Path>) -> Result<sqlx::SqlitePool> {
    let connection_options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let connection_options = connection_options.log_statements(LevelFilter::Debug);
    let pool = SqlitePoolOptions::default()
        .connect_with(connection_options)
        .await?;
    migration::migrate(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect() {
        let dir = tempfile::tempdir().unwrap();
        let pool = connect_sqlite(dir.path().join("test.sqlite"))
            .await
            .unwrap();
        assert!(!pool.is_closed());
        pool.close().await;
        assert!(pool.is_closed());
    }
}
