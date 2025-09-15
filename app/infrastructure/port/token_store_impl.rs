use domain::{iam::error::IamError, shared::port::token_store::TokenStoreTrait};
use kvdb_kit::{Kvdb, KvdbTrait as _};
use nject::injectable;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
#[injectable]
pub struct TokenStoreImpl {
    kvdb: Kvdb,
}
impl TokenStoreTrait for TokenStoreImpl {
    type Error = IamError;

    #[tracing::instrument]
    async fn store(
        &self,
        key: String,
        token: String,
        ex_at: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let key = Self::fill_key(&key);
        self.kvdb
            .set_with_ex_at(&key, token, ex_at.timestamp())
            .await
            .map_err(|_| IamError::AccessTokenSaveFailed)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn retrieve(&self, key: String) -> Option<String> {
        let full_key = Self::fill_key(&key);
        self.kvdb.get::<String>(&full_key).await
    }

    #[tracing::instrument]
    async fn delete(&self, key: String) -> Result<(), Self::Error> {
        let full_key = Self::fill_key(&key);
        let _ = self.kvdb.delete(&full_key).await;
        Ok(())
    }
}

impl TokenStoreImpl {
    fn fill_key(key: &str) -> String {
        format!("user:access_token:{key}")
    }
}
