use anyhow::Result;
use background_job::Job;
use infrastructure::{
    shared::kv::{Kv, KvTrait as _},
    shared::provider::Provider,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteExpiredKvJob;

impl Job for DeleteExpiredKvJob {
    type State = Provider;

    const NAME: &'static str = "delete_expired_kv_job";

    const CONCURRENCY: usize = 1;

    const RETRIES: usize = 0;

    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    async fn execute(_job: Self, state: &Self::State) -> Result<()> {
        let kv = state.provide::<Kv>();
        info!("Start delete expired kv job");
        let _ = kv.delete_expired();
        Ok(())
    }
}
