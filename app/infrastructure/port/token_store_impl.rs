use bon::Builder;
use domain::{shared::port::token_store::TokenStoreTrait, system::error::SystemError};
use kvdb_kit::{Kvdb, KvdbTrait as _};
use nject::injectable;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct TokenStoreImpl {
    kvdb: Kvdb,
}
impl TokenStoreTrait for TokenStoreImpl {
    type Error = SystemError;

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
            .map_err(|_| SystemError::AccessTokenSaveFailed)?;
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

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_kvdb;

    use super::*;
    use rstest::*;

    #[fixture]
    async fn token_store() -> TokenStoreImpl {
        let kvdb = setup_kvdb().await;
        TokenStoreImpl::builder().kvdb(kvdb).build()
    }

    #[rstest]
    #[tokio::test]
    async fn test_store(#[future(awt)] token_store: TokenStoreImpl) {
        token_store
            .store("test".to_string(), "token".to_string(), Utc::now())
            .await
            .unwrap();
        let token = token_store.retrieve("test".to_string()).await.unwrap();
        assert_eq!(token, "token");
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete(#[future(awt)] token_store: TokenStoreImpl) {
        token_store
            .store("test".to_string(), "token".to_string(), Utc::now())
            .await
            .unwrap();
        token_store.delete("test".to_string()).await.unwrap();
        let token = token_store.retrieve("test".to_string()).await;
        assert!(token.is_none());
    }
}
