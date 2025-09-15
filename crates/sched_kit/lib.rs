use anyhow::Result;

pub mod tokio_cron;

pub trait ScheduledJob: Clone + Send + Sync + 'static {
    const SCHEDULER: &'static str;
    fn run(&self) -> impl Future<Output = Result<()>> + Send;
}
