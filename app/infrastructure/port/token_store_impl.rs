use domain::{iam::error::IamError, shared::port::token_store::TokenStoreTrait};
use nject::injectable;
use sqlx::types::chrono::{DateTime, Utc};

use crate::shared::kv::{Kv, KvTrait};

#[derive(Clone)]
#[injectable]
pub struct TokenStoreImpl {
    kv: Kv,
}
impl TokenStoreTrait for TokenStoreImpl {
    type Error = IamError;
    async fn store(
        &self,
        key: String,
        token: String,
        ex_at: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let key = Self::fill_key(&key);
        self.kv
            .set_with_ex_at(&key, token, ex_at.timestamp())
            .await
            .map_err(|_| IamError::AccessTokenSaveFailed)?;
        Ok(())
    }
    async fn retrieve(&self, key: String) -> Option<String> {
        let full_key = Self::fill_key(&key);
        self.kv.get::<String>(&full_key).await
    }
    async fn delete(&self, key: String) -> Result<(), Self::Error> {
        let full_key = Self::fill_key(&key);
        let _ = self.kv.delete(&full_key).await;
        Ok(())
    }
}

impl TokenStoreImpl {
    fn fill_key(key: &str) -> String {
        format!("user:access_token:{key}")
    }
}
