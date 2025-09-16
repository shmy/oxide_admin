use anyhow::Result;
use futures_util::StreamExt as _;
use nject::injectable;
use sched_kit::ScheduledJob;

use crate::system::service::{file_service::FileService, upload_service::UploadService};

#[derive(Clone)]
#[injectable]
pub struct CleanupUnusedFile {
    file_service: FileService,
    upload_service: UploadService,
}

/// 每日凌晨1点删除未使用的文件
impl ScheduledJob for CleanupUnusedFile {
    const SCHEDULER: &'static str = "at 01:01 every day";

    async fn run(&self) -> Result<()> {
        let file_service = &self.file_service;
        let upload_service = &self.upload_service;

        // 收集所有需要删除的文件
        let mut paths = Vec::new();
        let mut stream = file_service.unused_2days_ago();
        while let Some(Ok(row)) = stream.next().await {
            paths.push(row.path);
        }

        if paths.is_empty() {
            tracing::info!("No unused files found");
            return Ok(());
        }

        // 先批量标记/删除数据库记录
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
