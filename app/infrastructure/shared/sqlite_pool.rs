use anyhow::Result;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::path::Path;
use tracing::log::LevelFilter;

pub type SqlitePool = sqlx::Pool<sqlx::Sqlite>;

pub async fn try_new(path: impl AsRef<Path>) -> Result<sqlx::SqlitePool> {
    let connection_options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let connection_options = connection_options.log_statements(LevelFilter::Trace);

    let pool = SqlitePoolOptions::default()
        .connect_with(connection_options)
        .await?;
    Ok(pool)
}
