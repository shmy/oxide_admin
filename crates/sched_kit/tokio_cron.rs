use anyhow::{Ok, Result};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::error;

use crate::ScheduledJob;

pub struct TokioCronScheduler {
    sched: tokio_cron_scheduler::JobScheduler,
}

impl TokioCronScheduler {
    pub async fn try_new() -> Result<Self> {
        let sched = JobScheduler::new().await?;
        Ok(Self { sched })
    }

    pub async fn add<T: ScheduledJob>(&self, job: T, timezone: chrono_tz::Tz) -> Result<()> {
        self.sched
            .add(Job::new_async_tz(
                T::SCHEDULER,
                timezone,
                move |_uuid, _l| {
                    let job = job.clone();
                    Box::pin(async move {
                        if let Err(err) = job.run().await {
                            error!(%err, "Failed to run job");
                        }
                    })
                },
            )?)
            .await?;
        Ok(())
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        self.sched.start().await?;
        signal.await;
        self.sched.shutdown().await?;
        Ok(())
    }
}
