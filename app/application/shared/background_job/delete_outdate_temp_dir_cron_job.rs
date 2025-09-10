use std::time::{Duration, SystemTime};

use anyhow::Result;
use faktory_bg::{JobRunner, error::RunnerError};
use futures_util::StreamExt as _;
use infrastructure::shared::path::TEMP_DIR;
use nject::injectable;
use tokio::fs;
use tracing::warn;

#[derive(Clone)]
#[injectable]
pub struct DeleteOutdateTempDirCronJob;

impl JobRunner for DeleteOutdateTempDirCronJob {
    type Params = ();
    async fn run(&self, _params: Self::Params) -> Result<(), RunnerError> {
        let now = SystemTime::now();

        if let Ok(dir) = fs::read_dir(TEMP_DIR.as_path()).await {
            // 直接流式遍历，无需 Vec 缓存
            let stream = tokio_stream::wrappers::ReadDirStream::new(dir);

            stream
                .for_each_concurrent(8, |entry| async {
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
