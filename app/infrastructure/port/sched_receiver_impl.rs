use domain::{
    shared::port::domain_repository::DomainRepository as _,
    system::{entity::sched::Sched, value_object::sched_id::SchedId},
};
use nject::injectable;

use crate::repository::system::sched_repository::SchedRepositoryImpl;

#[derive(Clone)]
#[injectable]
pub struct SchedReceiverImpl {
    repository: SchedRepositoryImpl,
}

impl sched_kit::ScheduledJobReceiver for SchedReceiverImpl {
    async fn receive(&self, params: sched_kit::JobCallbackParams) {
        let sched = Sched::builder()
            .id(SchedId::generate())
            .key(params.key)
            .name(params.name)
            .schedule(params.schedule)
            .succeed(params.succeed)
            .result(params.result)
            .run_at(params.run_at)
            .duration_ms(params.duration_ms)
            .build();
        if let Err(err) = self.repository.save(sched).await {
            tracing::error!(error = %err, "Failed to save sched");
        }
    }
}
