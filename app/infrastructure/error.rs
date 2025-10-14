use domain::{
    organization::{error::OrganizationError, value_object::hashed_password::PasswordError},
    system::error::SystemError,
};
use kvdb_kit::error::KvdbError;

pub type InfrastructureResult<T> = std::result::Result<T, InfrastructureError>;

#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    System(#[from] SystemError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    Password(#[from] PasswordError),

    #[error(transparent)]
    Kvdb(#[from] KvdbError),

    #[error(transparent)]
    Flag(#[from] flag_kit::error::FlagError),

    #[error(transparent)]
    Migrate(#[from] migrate_kit::error::MigrateError),
}
