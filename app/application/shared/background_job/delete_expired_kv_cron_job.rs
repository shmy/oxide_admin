use faktory_bg::{JobRunner, error::RunnerError};
use infrastructure::shared::kv::{Kv, KvTrait as _};
use nject::injectable;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct DeleteExpiredKvCronJob {
    kv: Kv,
}

impl JobRunner for DeleteExpiredKvCronJob {
    type Params = ();
    async fn run(&self, _params: Self::Params) -> Result<(), RunnerError> {
        info!("Start delete expired kv job");
        let _ = self.kv.delete_expired().await;
        Ok(())
    }
}
