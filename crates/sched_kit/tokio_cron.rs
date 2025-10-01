use crate::error::Result;
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

#[cfg(test)]
mod tests {

    use super::*;
    use std::result::Result::Ok;
    use tracing::info;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_add() {
        #[derive(Clone)]
        struct TestJob;

        impl ScheduledJob for TestJob {
            const SCHEDULER: &'static str = "every 1 second";
            const NAME: &'static str = "test job";

            async fn run(&self) -> Result<()> {
                info!("test job running");
                Ok(())
            }
        }
        let mut sched = TokioCronScheduler::try_new().await.unwrap();

        sched.add(TestJob, chrono_tz::Asia::Shanghai).await.unwrap();
        assert!(
            sched
                .run_with_signal(tokio::time::sleep(std::time::Duration::from_secs(2)))
                .await
                .is_ok()
        );
        assert!(logs_contain("test job running"));
    }
}
