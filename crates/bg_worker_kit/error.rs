pub type Result<T> = std::result::Result<T, WorkerError>;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[cfg(feature = "sqlite")]
    #[error("{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[cfg(feature = "sqlite")]
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[cfg(feature = "sqlite")]
    #[error("{0}")]
    Send(#[from] tokio::sync::broadcast::error::SendError<i64>),

    #[cfg(feature = "faktory")]
    #[error("Faktory error: {0}")]
    Faktory(#[from] faktory::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}
