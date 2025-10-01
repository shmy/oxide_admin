use crate::{JobCallbackParams, error::Result};
use chrono::Utc;
use tokio::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{ScheduledJob, ScheduledJobReceiver};
use std::future::Future;

pub struct TokioCronScheduler<R: ScheduledJobReceiver> {
    sched: tokio_cron_scheduler::JobScheduler,
    receiver: R,
}

impl<R: ScheduledJobReceiver> TokioCronScheduler<R> {
    pub async fn try_new(receiver: R) -> Result<Self> {
        let sched = JobScheduler::new().await?;
        Ok(Self { sched, receiver })
    }

    pub async fn add<T: ScheduledJob>(
        &self,
        key: &str,
        job: T,
        timezone: chrono_tz::Tz,
    ) -> Result<()> {
        let receiver = self.receiver.clone();
        let key = key.to_string();

        self.sched
            .add(Job::new_async_tz(
                T::SCHEDULER,
                timezone,
                move |_uuid, _l| {
                    let receiver = receiver.clone();
                    let job = job.clone();
                    let key = key.clone();
                    Box::pin(async move {
                        let now = Utc::now().with_timezone(&timezone).naive_local();
                        let instant = Instant::now();
                        let output = job.run().await;
                        let params = JobCallbackParams {
                            key,
                            name: T::NAME.to_string(),
                            schedule: T::SCHEDULER.to_string(),
                            succeed: output.is_ok(),
                            result: format!("{:?}", output),
                            run_at: now,
                            duration_ms: instant.elapsed().as_millis() as i64,
                        };
                        receiver.receive(params).await;
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
        #[derive(Clone)]
        struct TestJobReceiver;

        impl ScheduledJobReceiver for TestJobReceiver {
            async fn receive(&self, _params: JobCallbackParams) {}
        }

        let mut sched = TokioCronScheduler::try_new(TestJobReceiver).await.unwrap();

        sched
            .add("test_job", TestJob, chrono_tz::Asia::Shanghai)
            .await
            .unwrap();
        assert!(
            sched
                .run_with_signal(tokio::time::sleep(std::time::Duration::from_secs(2)))
                .await
                .is_ok()
        );
        assert!(logs_contain("test job running"));
    }
}
