pub type Result<T> = std::result::Result<T, WorkerError>;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Apalis(#[from] apalis::prelude::Error),

    #[error("Timeout")]
    Timeout,

    #[error(transparent)]
    BroadcastSend(#[from] tokio::sync::broadcast::error::SendError<i64>),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Custom(String),
}
