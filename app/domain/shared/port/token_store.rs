use sqlx::types::chrono::{DateTime, Utc};

pub trait TokenStoreTrait {
    type Error;
    fn store(
        &self,
        key: String,
        token: String,
        ex_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), Self::Error>>;
    fn retrieve(&self, key: String) -> impl Future<Output = Option<String>>;
    fn delete(&self, key: String) -> impl Future<Output = Result<(), Self::Error>>;
}
