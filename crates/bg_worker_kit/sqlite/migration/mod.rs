use crate::error::Result;

pub async fn migrate(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::migrate!("sqlite/migration/sql").run(pool).await?;
    Ok(())
}
