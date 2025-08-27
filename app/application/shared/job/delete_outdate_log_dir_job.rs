use anyhow::Result;
use background_job::Job;
use futures_util::StreamExt;
use infrastructure::{
    shared::provider::Provider,
    shared::{config::Config, path::LOG_DIR},
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOutdateLogDirJob;

impl Job for DeleteOutdateLogDirJob {
    type State = Provider;

    const NAME: &'static str = "delete_outdate_log_dir_job";
    const CONCURRENCY: usize = 1;
    const RETRIES: usize = 0;
    const TIMEOUT: Duration = Duration::from_secs(30);

    async fn execute(_job: Self, state: &Self::State) -> Result<()> {
        let config = state.provide::<Config>();
        let period_secs = config.log.rolling_period.as_secs();
        let now = SystemTime::now();

        if let Ok(dir) = fs::read_dir(LOG_DIR.as_path()).await {
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
                                if age.as_secs() > period_secs
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
