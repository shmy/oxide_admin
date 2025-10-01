use nject::injectable;

use crate::shared::pg_pool::PgPool;

#[derive(Clone)]
#[injectable]
pub struct SchedReceiverImpl {
    pool: PgPool,
}

impl sched_kit::ScheduledJobReceiver for SchedReceiverImpl {
    async fn receive(&self, params: sched_kit::JobCallbackParams) {
        dbg!(params);
    }
}
