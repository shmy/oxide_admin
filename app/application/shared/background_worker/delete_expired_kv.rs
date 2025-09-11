use bg_worker::{JobRunner, error::RunnerError};
use kvdb::{Kvdb, KvdbTrait as _};
use nject::injectable;
use tracing::info;

#[derive(Clone)]
#[injectable]
pub struct DeleteExpiredKv {
    kvdb: Kvdb,
}

impl JobRunner for DeleteExpiredKv {
    type Params = ();
    async fn run(&self, _params: Self::Params) -> Result<(), RunnerError> {
        info!("Start delete expired kv job");
        let _ = self.kvdb.delete_expired().await;
        Ok(())
    }
}
