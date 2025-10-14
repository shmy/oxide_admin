use cache_kit::error::CacheError;
use domain::{
    auth::error::AuthError,
    organization::{error::OrganizationError, value_object::hashed_password::PasswordError},
    system::error::SystemError,
};
use infrastructure::error::InfrastructureError;
use kvdb_kit::error::KvdbError;
use object_storage_kit::error::ObjectStorageError;
use sched_kit::error::SchedError;
use thiserror::Error;

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("unsupported_image_format")]
    UnsupportedImageFormat,

    #[error("illegal_token")]
    IllegalToken,

    #[error("recycled_token")]
    RecycledToken,

    #[error("permission_denied")]
    PermissionDenied,

    #[error(transparent)]
    System(#[from] SystemError),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),

    #[error(transparent)]
    Infrastructure(#[from] InfrastructureError),

    #[error(transparent)]
    ObjectStorage(#[from] ObjectStorageError),

    #[error(transparent)]
    Kvdb(#[from] KvdbError),

    #[error(transparent)]
    Sched(#[from] SchedError),

    #[error(transparent)]
    Cache(#[from] CacheError),

    #[error(transparent)]
    Password(#[from] PasswordError),
}
