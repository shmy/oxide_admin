#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("Faktory error: {0}")]
    Faktory(#[from] faktory::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}
