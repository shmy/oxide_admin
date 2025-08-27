use std::{fmt, sync::Arc};

#[derive(Debug, Clone)]
pub struct CloneableError(Arc<anyhow::Error>);

impl fmt::Display for CloneableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CloneableError {}

impl From<sqlx::Error> for CloneableError {
    fn from(value: sqlx::Error) -> Self {
        Self(Arc::new(value.into()))
    }
}

impl From<anyhow::Error> for CloneableError {
    fn from(value: anyhow::Error) -> Self {
        Self(Arc::new(value))
    }
}
