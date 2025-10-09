pub type Result<T> = std::result::Result<T, MigrateError>;
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
