#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SystemError {
    #[error("File not exists")]
    FileNotFound,
    #[error("Sched not exists")]
    SchedNotFound,
    #[error("{0}")]
    Sqlx(String),
    #[error("AccessLog not exists")]
    AccessLogNotFound,
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
