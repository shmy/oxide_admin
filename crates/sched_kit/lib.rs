use chrono::NaiveDateTime;

use crate::error::Result;

pub mod error;
pub mod tokio_cron;

pub trait ScheduledJob: Clone + Send + Sync + 'static {
    const SCHEDULER: &'static str;
    const NAME: &'static str;
    fn run(&self) -> impl Future<Output = Result<()>> + Send;
}

pub trait ScheduledJobReceiver: Clone + Send + Sync + 'static {
    fn receive(&self, params: JobCallbackParams) -> impl Future<Output = ()> + Send;
}

#[derive(Clone, Debug)]
pub struct JobCallbackParams {
    pub key: String,
    pub name: String,
    pub schedule: String,
    pub succeed: bool,
    pub output: String,
    pub run_at: NaiveDateTime,
    pub duration_ms: i64,
}
