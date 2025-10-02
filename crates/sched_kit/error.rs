use cron_tab::CronError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SchedError>;

#[derive(Debug, Error)]
pub enum SchedError {
    #[error("Failed to convert to cron")]
    EnglishToCron,
    #[error("{0}")]
    Cron(#[from] CronError),
    #[error("{0}")]
    Custom(String),
}
