use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::types::chrono::{DateTime, Utc};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug)]
pub struct TokenIssuerOutput {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
}

pub trait TokenIssuerTrait {
    type Error: Send + Sync + Error + 'static;
    fn generate_access_token<T: Serialize>(
        &self,
        claims: &T,
        secret: &[u8],
    ) -> Result<String, Self::Error>;
    fn generate_refresh_token(&self) -> String;
    fn generate(&self, sub: String) -> Result<TokenIssuerOutput, Self::Error>;

    fn verify<T: DeserializeOwned + Clone>(
        &self,
        access_token: &str,
        secret: &[u8],
    ) -> Result<T, Self::Error>;
}
