use std::collections::HashMap;

use crate::error::Result;
use bon::Builder;
use sqlx::{Executor, FromRow, PgConnection, PgPool};

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: &'static str,
    pub content: &'static str,
    pub checksum: &'static str,
}
#[derive(Debug, Clone, FromRow)]
struct AppliedMigration {
    version: String,
    checksum: String,
}

#[derive(Builder)]
pub struct Migrator {
    pool: PgPool,
    #[builder(default = "_migrations")]
    table_name: &'static str,
}

impl Migrator {
    async fn get_applied(&self, pool: &PgPool) -> Result<HashMap<String, String>> {
        let rows: Vec<AppliedMigration> = sqlx::query_as(&format!(
            "SELECT version, checksum FROM {}",
            self.table_name
        ))
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.version, r.checksum)).collect())
    }

    async fn apply_migration(&self, conn: &mut PgConnection, migration: &Migration) -> Result<()> {
        conn.execute(&*migration.content).await?;
        sqlx::query(&format!(
            "INSERT INTO {}(version, checksum) VALUES($1, $2)",
            self.table_name,
        ))
        .bind(migration.version)
        .bind(migration.checksum)
        .execute(&mut *conn)
        .await?;
        tracing::info!("Applied migration: {}", migration.version);
        Ok(())
    }

    pub async fn migrate(&mut self, migrations: &[Migration]) -> Result<()> {
        sqlx::query(&format!(
            r#"
        CREATE TABLE IF NOT EXISTS {} (
            version VARCHAR(64) PRIMARY KEY,
            checksum VARCHAR(64) NOT NULL,
            applied_at TIMESTAMP NOT NULL DEFAULT now()
        )
        "#,
            self.table_name
        ))
        .execute(&self.pool)
        .await?;

        let applied = self.get_applied(&self.pool).await?;

        let pool = self.pool.clone();
        let mut tx = pool.begin().await?;
        for m in migrations {
            if let Some(checksum) = applied.get(m.version) {
                if checksum != m.checksum {
                    panic!(
                        "Migration {} checksum mismatch, local checksum={}, db checksum={:?}",
                        m.version, m.checksum, checksum
                    );
                }
            } else {
                self.apply_migration(&mut tx, &m).await?;
            }
        }
        tx.commit().await?;

        tracing::info!("All migrations applied.");
        Ok(())
    }
}
