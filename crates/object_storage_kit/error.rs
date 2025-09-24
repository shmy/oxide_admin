use thiserror::Error;

pub type Result<T> = std::result::Result<T, ObjectStorageError>;

#[derive(Debug, Error)]
pub enum ObjectStorageError {
    #[error("{0}")]
    Opendal(#[from] opendal::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
}
