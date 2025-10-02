use chrono::NaiveDateTime;

use crate::error::Result;

pub mod cron_tab;
pub mod error;

pub trait ScheduledJob: Clone + Send + Sync + 'static {
    const EXPR: &'static str;
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
    pub expr: String,
    pub succeed: bool,
    pub result: String,
    pub run_at: NaiveDateTime,
    pub duration_ms: i64,
}
