use anyhow::Result;
use domain::shared::id_generator::IdGenerator;
use futures_util::stream::BoxStream;
use infrastructure::{
    shared::chrono_tz::{ChronoTz, Duration},
    shared::pool::Pool,
};
use nject::injectable;
use sqlx::prelude::FromRow;

#[derive(Debug, sqlx::Type)]
#[repr(i16)]
pub enum FileStatus {
    Unused = 1,
    Used = 2,
}

#[derive(Clone)]
#[injectable]
pub struct FileService {
    ct: ChronoTz,
    pool: Pool,
}

impl FileService {
    pub fn unused_2days_ago(&self) -> BoxStream<'_, Result<File, sqlx::Error>> {
        let now = self.ct.now();
        let two_days_ago = now - Duration::days(2);

        (sqlx::query_as!(
            File,
            "SELECT path, status from _files WHERE status = $1 AND created_at < $2",
            FileStatus::Unused as FileStatus,
            two_days_ago
        )
        .fetch(&self.pool)) as _
    }

    pub async fn create(&self, relative_path: &str) -> Result<()> {
        let now = self.ct.now();
        let id = IdGenerator::primary_id();
        let _ = sqlx::query!(
            "INSERT INTO _files (id, path, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            id,
            relative_path,
            FileStatus::Unused as FileStatus,
            now,
            now,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_files_unused(&self, relative_paths: &[String]) -> Result<()> {
        self.set_files_status(relative_paths, FileStatus::Unused)
            .await
    }

    pub async fn set_files_used(&self, relative_paths: &[String]) -> Result<()> {
        self.set_files_status(relative_paths, FileStatus::Used)
            .await
    }

    async fn set_files_status(&self, relative_paths: &[String], status: FileStatus) -> Result<()> {
        if relative_paths.is_empty() {
            return Ok(());
        }
        let now = self.ct.now();
        sqlx::query!(
            r#"
            UPDATE _files SET status = $1, updated_at = $2 WHERE path = ANY($3)
            "#,
            status as FileStatus,
            now,
            relative_paths,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_files(&self, relative_paths: &[String]) -> Result<()> {
        if relative_paths.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            r#"
            DELETE FROM _files WHERE path = ANY($1)
            "#,
            relative_paths
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Clone, FromRow)]
pub struct File {
    pub path: String,
    pub status: i64,
}
