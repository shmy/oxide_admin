use domain::system::{error::SystemError, value_object::hashed_password::PasswordError};
use kvdb_kit::error::KvdbError;

pub type InfrastructureResult<T> = std::result::Result<T, InfrastructureError>;

#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("{0}")]
    System(#[from] SystemError),

    #[error("{0}")]
    Password(#[from] PasswordError),

    #[error("{0}")]
    Kvdb(#[from] KvdbError),

    #[error("{0}")]
    Flag(#[from] flag_kit::error::FlagError),
}
