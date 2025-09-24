use std::fmt::Debug;

use crate::error::Result;
use sqlx::ConnectOptions as _;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tracing::info;
use tracing::log::LevelFilter;

use crate::shared::config::Database;

pub type PgPool = sqlx::PgPool;

pub async fn try_new(timezone: chrono_tz::Tz, db: &Database) -> Result<PgPool> {
    let pool_connection_options: PgConnectOptions = db.url.parse()?;
    let pool_connection_options = pool_connection_options.log_statements(LevelFilter::Debug);

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
    info!("{} connected", row.version);
    info!("Database timezone: {}", row.timezone);
    Ok(pool)
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use testcontainers::{ImageExt, runners::AsyncRunner as _};
    use testcontainers_modules::postgres;

    use super::*;

    #[tokio::test]
    async fn test_try_new() {
        let container = postgres::Postgres::default()
            .with_tag("17-alpine")
            .start()
            .await
            .unwrap();
        let connection_string = format!(
            "postgresql://postgres:postgres@127.0.0.1:{}/postgres",
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let result = try_new(
            chrono_tz::Asia::Shanghai,
            &Database::builder()
                .url(connection_string)
                .max_connections(10)
                .min_connections(5)
                .max_lifetime(Duration::from_secs(60))
                .idle_timeout(Duration::from_secs(30))
                .acquire_timeout(Duration::from_secs(10))
                .build(),
        )
        .await;
        assert!(result.is_ok());
    }
}
