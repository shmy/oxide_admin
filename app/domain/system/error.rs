#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SystemError {
    #[error("file_not_found")]
    FileNotFound,
    #[error("sched_not_found")]
    SchedNotFound,
    #[error("accessLog_not_found")]
    AccessLogNotFound,
    #[error("database_error")]
    Sqlx(String),
}
impl From<sqlx::Error> for SystemError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(% err, "sqlx error");
        let message = err.to_string();
        Self::Sqlx(message)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_system_error() {
        let err = SystemError::from(sqlx::Error::RowNotFound);
        assert_eq!(err, SystemError::Sqlx(sqlx::Error::RowNotFound.to_string()));
    }
}
