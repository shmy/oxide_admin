use domain::{
    auth::error::AuthError, organization::error::OrganizationError, system::error::SystemError,
};
use infrastructure::error::InfrastructureError;
use kvdb_kit::error::KvdbError;
use object_storage_kit::error::ObjectStorageError;
use sched_kit::error::SchedError;
use thiserror::Error;

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Unsupported image format")]
    UnsupportedImageFormat,
    #[error("Illegal token")]
    IllegalToken,

    #[error("Recycled token")]
    RecycledToken,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("{0}")]
    System(#[from] SystemError),

    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("{0}")]
    Organization(#[from] OrganizationError),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Image(#[from] image::ImageError),

    #[error("{0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("{0}")]
    Persist(#[from] tempfile::PersistError),

    #[error("{0}")]
    Infrastructure(#[from] InfrastructureError),

    #[error("{0}")]
    ObjectStorage(#[from] ObjectStorageError),

    #[error("{0}")]
    Kvdb(#[from] KvdbError),

    #[error("{0}")]
    Sched(#[from] SchedError),
}
