use std::collections::HashSet;

use crate::error::Result;
use bon::Builder;
use include_dir::Dir;
use sqlx::{Executor, FromRow, PgConnection, PgPool};

#[derive(Debug, Clone)]
struct Migration {
    version: String,
    content: String,
}
#[derive(Debug, Clone, FromRow)]
struct AppliedMigration {
    version: String,
}

#[derive(Builder)]
pub struct Migrator {
    pool: PgPool,
    #[builder(default = "_migrations")]
    table_name: &'static str,
}

impl Migrator {
    async fn load_migrations(&self, dir: &Dir<'_>) -> Result<Vec<Migration>> {
        let mut migrations = vec![];

        for file in dir.files() {
            let name = file
                .path()
                .file_stem()
                .expect("File name must have a stem")
                .to_string_lossy()
                .to_string();
            let content = std::str::from_utf8(file.contents())
                .expect("File must be utf-8")
                .to_string();

            migrations.push(Migration {
                version: name,
                content,
            });
        }

        migrations.sort_by(|a, b| a.version.cmp(&b.version));
        Ok(migrations)
    }

    async fn get_applied_versions(&self, pool: &PgPool) -> Result<HashSet<String>> {
        let rows: Vec<AppliedMigration> =
            sqlx::query_as(&format!("SELECT version FROM {}", self.table_name))
                .fetch_all(pool)
                .await?;
        Ok(rows.into_iter().map(|r| r.version).collect())
    }

    async fn apply_migration(&self, conn: &mut PgConnection, migration: &Migration) -> Result<()> {
        conn.execute(&*migration.content).await?;
        sqlx::query(&format!(
            "INSERT INTO {}(version) VALUES($1)",
            self.table_name
        ))
        .bind(migration.version.to_string())
        .execute(&mut *conn)
        .await?;
        tracing::info!("Applied migration: {}", migration.version);
        Ok(())
    }

    pub async fn migrate(&mut self, dir: &Dir<'_>) -> Result<()> {
        sqlx::query(&format!(
            r#"
        CREATE TABLE IF NOT EXISTS {} (
            version VARCHAR(64) PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT now()
        )
        "#,
            self.table_name
        ))
        .execute(&self.pool)
        .await?;

        let migrations = self.load_migrations(dir).await?;
        let applied = self.get_applied_versions(&self.pool).await?;

        let pool = self.pool.clone();
        let mut tx = pool.begin().await?;
        for m in migrations {
            if !applied.contains(&m.version) {
                self.apply_migration(&mut tx, &m).await?;
            }
        }
        tx.commit().await?;

        tracing::info!("All migrations applied.");
        Ok(())
    }
}
