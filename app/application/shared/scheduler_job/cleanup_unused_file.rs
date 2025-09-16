use anyhow::Result;
use nject::injectable;
use sched_kit::ScheduledJob;

#[derive(Clone)]
#[injectable]
pub struct CleanupUnusedFile {}

/// 每日凌晨1点删除未使用的文件
impl ScheduledJob for CleanupUnusedFile {
    const SCHEDULER: &'static str = "at 12:20 every day";

    async fn run(&self) -> Result<()> {
        // let file_service = &self.file_service;
        // let stream = file_service.unused_2days_ago();
        // stream
        //     .for_each_concurrent(4, |row_ret| async {
        //         if let Ok(row) = row_ret {
        //             let abs_path = UPLOAD_DIR.join(&row.path);
        //             match file_service.delete_files(&[row.path]).await {
        //                 Ok(_) => {
        //                     if let Err(err) = fs::remove_file(&abs_path).await {
        //                         tracing::error!(?err, "删除文件失败: {:?}", abs_path);
        //                     }
        //                 }
        //                 Err(err) => {
        //                     tracing::error!(?err, "标记文件删除失败: {:?}", abs_path);
        //                 }
        //             }
        //         }
        //     })
        //     .await;
        todo!();
        Ok(())
    }
}
