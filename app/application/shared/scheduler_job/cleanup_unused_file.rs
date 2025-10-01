use bon::Builder;
use futures_util::StreamExt as _;
use nject::injectable;
use sched_kit::{ScheduledJob, error::Result};

use crate::system::service::{file_service::FileService, upload_service::UploadService};

#[derive(Clone, Builder)]
#[injectable]
pub struct CleanupUnusedFile {
    file_service: FileService,
    upload_service: UploadService,
}

/// Delete unused files at 1 a.m. every day.
impl ScheduledJob for CleanupUnusedFile {
    const SCHEDULER: &'static str = "at 01:01 every day";
    const NAME: &'static str = "Delete unused files";

    async fn run(&self) -> Result<()> {
        let file_service = &self.file_service;
        let upload_service = &self.upload_service;

        let mut paths = Vec::new();
        let mut stream = file_service.unused_2days_ago();
        while let Some(Ok(row)) = stream.next().await {
            paths.push(row.path);
        }

        if paths.is_empty() {
            tracing::info!("No unused files found");
            return Ok(());
        }

        match file_service.delete_files(&paths).await {
            Ok(_) => {
                tracing::info!("Deleting {} unused files", paths.len());
                if let Err(err) = upload_service.delete_many(paths).await {
                    tracing::error!(%err, "Delete file failed");
                }
            }
            Err(err) => {
                tracing::error!(%err, "Mark file as unused failed");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use domain::shared::id_generator::IdGenerator;
    use infrastructure::{
        shared::{
            chrono_tz::{ChronoTz, Duration},
            pg_pool::PgPool,
            workspace::WorkspaceRef,
        },
        test_utils::{setup_database, setup_object_storage},
    };
    use sqlx::types::chrono::Utc;

    use super::*;

    async fn build_job(pool: PgPool) -> CleanupUnusedFile {
        setup_database(pool.clone()).await;
        let object_storage = setup_object_storage().await;
        let file_service = FileService::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let upload_service = {
            let file_service = {
                FileService::builder()
                    .pool(pool.clone())
                    .ct(ChronoTz::default())
                    .build()
            };
            UploadService::builder()
                .ct(ChronoTz::default())
                .object_storage(object_storage)
                .file_service(file_service)
                .workspace(WorkspaceRef::default())
                .build()
        };
        CleanupUnusedFile::builder()
            .file_service(file_service)
            .upload_service(upload_service)
            .build()
    }

    #[sqlx::test]
    async fn test_cleanup_unused_files(pool: PgPool) {
        let job = build_job(pool.clone()).await;
        assert!(job.run().await.is_ok());
        // Insert a 2-day-old file
        let now = Utc::now() - Duration::days(3);
        let id = IdGenerator::primary_id();
        let insert = sqlx::query("INSERT INTO _files (id, name, size, path, used, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(id)
            .bind("test3.txt")
            .bind(125)
            .bind("/test/test3.txt")
            .bind(false)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await;
        assert!(insert.is_ok());
        assert!(job.run().await.is_ok());
    }
}
