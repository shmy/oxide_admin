use anyhow::Result;
use background_job::CronJob;
use futures_util::StreamExt as _;
use infrastructure::shared::path::UPLOAD_DIR;
use nject::injectable;
use tokio::fs;

use crate::system::service::file_service::FileService;

#[derive(Clone)]
#[injectable]
pub struct DeleteUnusedFileCronJob {
    file_service: FileService,
}

impl CronJob for DeleteUnusedFileCronJob {
    const NAME: &'static str = "delete_unused_file_cron_job";
    const SCHEDULE: &'static str = "at 00:01 am";

    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    async fn execute(&self) -> Result<()> {
        let file_service = &self.file_service;
        let stream = file_service.unused_2days_ago();
        stream
            .for_each_concurrent(4, |row_ret| async {
                if let Ok(row) = row_ret {
                    let abs_path = UPLOAD_DIR.join(&row.path);
                    match file_service.delete_files(&[row.path]).await {
                        Ok(_) => {
                            if let Err(err) = fs::remove_file(&abs_path).await {
                                tracing::error!(?err, "删除文件失败: {:?}", abs_path);
                            }
                        }
                        Err(err) => {
                            tracing::error!(?err, "标记文件删除失败: {:?}", abs_path);
                        }
                    }
                }
            })
            .await;
        Ok(())
    }
}
