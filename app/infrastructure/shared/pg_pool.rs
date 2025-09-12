use std::fmt::Debug;

use anyhow::Result;
use sqlx::ConnectOptions as _;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tracing::info;
use tracing::log::LevelFilter;

use crate::shared::config::Database;

pub type PgPool = sqlx::PgPool;

pub async fn try_new(db: &Database) -> Result<PgPool> {
    let pool_connection_options: PgConnectOptions = db.url.parse()?;
    let pool_connection_options = pool_connection_options.log_statements(LevelFilter::Debug);

    let timezone = db.timezone;
    let pool = PgPoolOptions::new()
        .max_connections(db.max_connections)
        .min_connections(db.min_connections)
        .max_lifetime(db.max_lifetime)
        .idle_timeout(db.idle_timeout)
        .acquire_timeout(db.acquire_timeout)
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                let sql = format!("SET TIME ZONE '{timezone}'");
                sqlx::query(&sql).execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(pool_connection_options)
        .await?;
    let row = sqlx::query!(
        r#"
        SELECT version() AS "version!", current_setting('TimeZone') AS "timezone!"
        "#
    )
    .fetch_one(&pool)
    .await?;
    info!("{}", row.version);
    info!("Database timezone: {}", row.timezone);
    Ok(pool)
}
