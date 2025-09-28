use bon::Builder;
use futures_util::StreamExt as _;
use infrastructure::shared::workspace::WorkspaceRef;
use nject::injectable;
use sched_kit::{ScheduledJob, error::Result};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::warn;

#[derive(Clone, Builder)]
#[injectable]
pub struct CleanupTempDir {
    workspace: WorkspaceRef,
}

/// 每日凌晨0点删除两天前的临时目录
impl ScheduledJob for CleanupTempDir {
    const SCHEDULER: &'static str = "at 00:01 every day";

    async fn run(&self) -> Result<()> {
        let now = SystemTime::now();

        if let Ok(dir) = fs::read_dir(self.workspace.temp_dir()).await {
            // 直接流式遍历，无需 Vec 缓存
            let stream = tokio_stream::wrappers::ReadDirStream::new(dir);

            stream
                .for_each_concurrent(4, |entry| async {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if !path.is_dir() {
                                return;
                            }

                            if let Ok(meta) = fs::metadata(&path).await
                                && let Ok(modified) = meta.modified()
                            {
                                let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
                                if age.as_secs() > 2 * 24 * 3600
                                    && let Err(err) = fs::remove_dir_all(&path).await
                                {
                                    warn!(
                                        "Failed to delete outdated log dir: {} {}",
                                        path.display(),
                                        err
                                    );
                                }
                            }
                        }
                        Err(err) => warn!("Read dir entry failed: {}", err),
                    }
                })
                .await;
        }

        Ok(())
    }
}
