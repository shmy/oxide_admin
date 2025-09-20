use anyhow::Result;
use bon::Builder;
use domain::shared::id_generator::IdGenerator;
use futures_util::stream::BoxStream;
use infrastructure::shared::{
    chrono_tz::{ChronoTz, Duration},
    pg_pool::PgPool,
};
use nject::injectable;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct FileService {
    ct: ChronoTz,
    pool: PgPool,
}

impl FileService {
    #[tracing::instrument]
    pub fn unused_2days_ago(&self) -> BoxStream<'_, Result<File, sqlx::Error>> {
        let now = self.ct.now();
        let two_days_ago = now - Duration::days(2);

        (sqlx::query_as!(
            File,
            "SELECT path, used from _files WHERE used = false AND created_at < $1",
            two_days_ago
        )
        .fetch(&self.pool)) as _
    }

    #[tracing::instrument]
    pub async fn create(&self, relative_path: &str) -> Result<()> {
        let now = self.ct.now();
        let id = IdGenerator::primary_id();
        let _ = sqlx::query!(
            "INSERT INTO _files (id, path, used, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            id,
            relative_path,
            false,
            now,
            now,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument]
    pub async fn set_files_unused(&self, relative_paths: &[String]) -> Result<()> {
        self.set_files_status(relative_paths, false).await
    }

    #[tracing::instrument]
    pub async fn set_files_used(&self, relative_paths: &[String]) -> Result<()> {
        self.set_files_status(relative_paths, true).await
    }

    #[tracing::instrument]
    async fn set_files_status(&self, relative_paths: &[String], used: bool) -> Result<()> {
        if relative_paths.is_empty() {
            return Ok(());
        }
        let now = self.ct.now();
        sqlx::query!(
            r#"
            UPDATE _files SET used = $1, updated_at = $2 WHERE path = ANY($3)
            "#,
            used,
            now,
            relative_paths,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument]
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
    pub used: bool,
}

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt as _;
    use infrastructure::test_utils::setup_database;

    use super::*;

    async fn build_service(pool: PgPool) -> FileService {
        setup_database(pool.clone()).await;
        let service = FileService::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        assert!(service.create("/test/test1.txt").await.is_ok());
        assert!(service.create("/test/test2.txt").await.is_ok());
        service
    }

    #[derive(FromRow)]
    struct FileRow {
        used: bool,
    }

    #[sqlx::test]
    async fn test_files_unused_2days_ago(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let now = service.ct.now() - Duration::days(2);
        let id = IdGenerator::primary_id();
        let insert = sqlx::query("INSERT INTO _files (id, path, used, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(id)
            .bind("/test/test3.txt")
            .bind(false)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await;
        assert!(insert.is_ok());
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(files.len(), 3);
        let stream = service.unused_2days_ago();
        let files: Vec<_> = stream.try_collect().await.unwrap();
        assert_eq!(files.len(), 1);
    }

    #[sqlx::test]
    async fn test_files_toggle_used(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(files.len(), 2);
        for file in files {
            assert!(!file.used);
        }
        assert!(
            service
                .set_files_used(&["/test/test1.txt".to_string(), "/test/test2.txt".to_string(),])
                .await
                .is_ok()
        );
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        for file in files {
            assert!(file.used);
        }

        assert!(
            service
                .set_files_unused(&["/test/test1.txt".to_string(), "/test/test2.txt".to_string(),])
                .await
                .is_ok()
        );
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT path, used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        for file in files {
            assert!(!file.used);
        }
    }

    #[sqlx::test]
    async fn delete_files(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(files.len(), 2);
        assert!(
            service
                .delete_files(&["/test/test1.txt".to_string(), "/test/test2.txt".to_string(),])
                .await
                .is_ok()
        );
        let files: Vec<FileRow> = sqlx::query_as(r#"SELECT used FROM _files"#)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert!(files.is_empty());
    }
}
