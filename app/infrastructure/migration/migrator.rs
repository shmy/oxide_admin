use crate::error::InfrastructureResult;
use include_dir::Dir;
use include_dir::include_dir;
use sqlx::Executor;
use sqlx::{PgConnection, PgPool, prelude::FromRow};
use std::collections::HashSet;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migration/sql");

#[derive(Debug, Clone)]
struct Migration {
    version: String,
    content: String,
}
#[derive(Debug, Clone, FromRow)]
struct AppliedMigration {
    version: String,
}

async fn load_migrations() -> InfrastructureResult<Vec<Migration>> {
    let mut migrations = vec![];

    for file in MIGRATIONS_DIR.files() {
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

async fn get_applied_versions(pool: &PgPool) -> InfrastructureResult<HashSet<String>> {
    let rows: Vec<AppliedMigration> = sqlx::query_as("SELECT version FROM _migrations")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.version).collect())
}

async fn apply_migration(
    conn: &mut PgConnection,
    migration: &Migration,
) -> InfrastructureResult<()> {
    conn.execute(&*migration.content).await?;
    sqlx::query("INSERT INTO _migrations(version) VALUES($1)")
        .bind(migration.version.to_string())
        .execute(&mut *conn)
        .await?;
    tracing::info!("Applied migration: {}", migration.version);
    Ok(())
}

pub struct Migrator {
    pool: PgPool,
}

impl Migrator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&mut self) -> InfrastructureResult<()> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version VARCHAR(64) PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT now()
        )
        "#,
        )
        .execute(&self.pool)
        .await?;

        let migrations = load_migrations().await?;
        let applied = get_applied_versions(&self.pool).await?;

        let pool = self.pool.clone();
        let mut tx = pool.begin().await?;
        for m in migrations {
            if !applied.contains(&m.version) {
                apply_migration(&mut *tx, &m).await?;
            }
        }
        tx.commit().await?;

        tracing::info!("All migrations applied.");
        Ok(())
    }
}
