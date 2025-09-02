use anyhow::Result;
use background_job::CronJob;
use infrastructure::shared::kv::{Kv, KvTrait as _};
use nject::injectable;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct DeleteExpiredKvCronJob {
    kv: Kv,
}

impl CronJob for DeleteExpiredKvCronJob {
    const NAME: &'static str = "delete_expired_kv_cron_job";
    const SCHEDULE: &'static str = "every 1 hour";
    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    async fn execute(&self) -> Result<()> {
        info!("Start delete expired kv job");
        let _ = self.kv.delete_expired();
        Ok(())
    }
}
