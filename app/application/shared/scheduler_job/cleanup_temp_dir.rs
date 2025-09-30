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

/// Delete the temporary directory from two days ago at midnight every day.
impl ScheduledJob for CleanupTempDir {
    const SCHEDULER: &'static str = "at 00:01 every day";

    async fn run(&self) -> Result<()> {
        let now = SystemTime::now();

        if let Ok(dir) = fs::read_dir(self.workspace.temp_dir()).await {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_cleanup_temp_dir() {
        let workspace = WorkspaceRef::default();
        let temp_subdir = workspace.temp_dir().join("old_dir");
        std::fs::create_dir_all(&temp_subdir).unwrap();
        let path = temp_subdir.join("test.txt");
        std::fs::write(&path, "test").unwrap();
        assert!(path.exists());

        // Change the modification time to two days ago
        let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 24 * 3600 + 1);
        let ft = filetime::FileTime::from_system_time(two_days_ago);
        filetime::set_file_mtime(&temp_subdir, ft).unwrap();

        let job = CleanupTempDir::builder().workspace(workspace).build();
        assert!(job.run().await.is_ok());
        assert!(!path.exists());
    }
}
