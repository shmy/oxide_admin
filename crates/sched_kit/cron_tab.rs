use std::str::FromStr as _;

use crate::error::{Result, SchedError};
use crate::{JobCallbackParams, ScheduledJob, ScheduledJobReceiver};
use chrono::{DateTime, Utc};
use cron_tab::AsyncCron;
use tokio::time::Instant;
use tracing::info;

pub struct CronTab<R: ScheduledJobReceiver> {
    cron: AsyncCron<chrono_tz::Tz>,
    timezone: chrono_tz::Tz,
    receiver: R,
}

impl<R: ScheduledJobReceiver> CronTab<R> {
    pub fn new(receiver: R, timezone: chrono_tz::Tz) -> Self {
        let cron = AsyncCron::new(timezone);
        Self {
            cron,
            timezone,
            receiver,
        }
    }

    pub async fn add<T: ScheduledJob>(&mut self, key: &str, job: T) -> Result<()> {
        let cron_expr =
            english_to_cron::str_cron_syntax(T::EXPR).map_err(|_| SchedError::EnglishToCron)?;
        let key = key.to_string();
        let timezone = self.timezone;
        let receiver = self.receiver.clone();
        self.cron
            .add_fn(&cron_expr, move || {
                let key = key.clone();
                let job = job.clone();
                let receiver = receiver.clone();
                async move {
                    let now = Utc::now().with_timezone(&timezone).naive_local();
                    let instant = Instant::now();
                    let output = job.run().await;

                    let params = JobCallbackParams {
                        key,
                        name: T::NAME.to_string(),
                        expr: T::EXPR.to_string(),
                        succeed: output.is_ok(),
                        result: format!("{:?}", output),
                        run_at: now,
                        duration_ms: instant.elapsed().as_millis() as i64,
                    };
                    receiver.receive(params).await;
                }
            })
            .await?;
        let next_tick = next_tick_expr(&cron_expr, timezone);
        info!("[{}] registerd. next tick: {:?}", T::NAME, next_tick);

        Ok(())
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        self.cron.start().await;
        signal.await;
        self.cron.stop().await;
        Ok(())
    }
}

fn next_tick_expr(expr: &str, timezone: chrono_tz::Tz) -> Option<DateTime<chrono_tz::Tz>> {
    if let Ok(schedule) = cron::Schedule::from_str(expr) {
        return schedule.upcoming(timezone).next();
    }

    None
}

pub fn next_tick(expr: &str, timezone: chrono_tz::Tz) -> Option<DateTime<chrono_tz::Tz>> {
    if let Ok(cron_expr) = english_to_cron::str_cron_syntax(expr) {
        return next_tick_expr(&cron_expr, timezone);
    }

    None
}
