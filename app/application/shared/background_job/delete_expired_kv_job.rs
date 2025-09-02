use anyhow::Result;
use background_job::Job;
use infrastructure::shared::kv::{Kv, KvTrait as _};
use nject::injectable;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Serialize, Deserialize)]
pub struct DeleteExpiredKvJobParams;

#[derive(Clone)]
#[injectable]
pub struct DeleteExpiredKvJob {
    kv: Kv,
}

impl Job for DeleteExpiredKvJob {
    type Params = DeleteExpiredKvJobParams;

    const NAME: &'static str = "delete_expired_kv_job";

    const CONCURRENCY: usize = 1;

    const RETRIES: usize = 0;

    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    async fn execute(&self, _params: Self::Params) -> Result<()> {
        info!("Start delete expired kv job");
        let _ = self.kv.delete_expired();
        Ok(())
    }
}
