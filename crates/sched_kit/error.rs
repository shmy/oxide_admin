use thiserror::Error;
use tokio_cron_scheduler::JobSchedulerError;

pub type Result<T> = std::result::Result<T, SchedError>;

#[derive(Debug, Error)]
pub enum SchedError {
    #[error("{0}")]
    JobScheduler(#[from] JobSchedulerError),
    #[error("{0}")]
    Custom(String),
}
