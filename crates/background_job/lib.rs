pub use apalis::prelude::{Data, Error, GoTo, StepBuilder, SteppableStorage, Storage};
pub use apalis_core::codec::json::JsonCodec;
pub use apalis_core::step::StepFn;

#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "redis")]
pub use redis::*;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::*;

#[cfg(feature = "redis")]
pub type JobPool = bb8_redis::bb8::Pool<bb8_redis::RedisConnectionManager>;

#[cfg(feature = "sqlite")]
pub type JobPool = apalis_sql::sqlx::Pool<apalis_sql::sqlx::Sqlite>;

#[cfg(feature = "sqlite")]
pub async fn try_new(
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<apalis_sql::sqlx::SqlitePool> {
    use apalis_sql::sqlx::ConnectOptions;
    use apalis_sql::sqlx::sqlite::SqliteConnectOptions;
    use apalis_sql::sqlx::sqlite::SqliteJournalMode;
    use apalis_sql::sqlx::sqlite::SqlitePoolOptions;
    use tracing::log::LevelFilter;
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
