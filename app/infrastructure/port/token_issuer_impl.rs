use bon::Builder;
use domain::{
    auth::error::AuthError,
    auth::port::token_issuer::{TokenIssuerOutput, TokenIssuerTrait, UserClaims},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use nject::injectable;
use serde::{Serialize, de::DeserializeOwned};

use crate::shared::{chrono_tz::ChronoTz, config::ConfigRef};

const ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct TokenIssuerImpl {
    config: ConfigRef,
    ct: ChronoTz,
}

impl TokenIssuerTrait for TokenIssuerImpl {
    type Error = AuthError;

    #[tracing::instrument(skip(claims, secret))]
    fn generate_access_token<T: Serialize>(
        &self,
        claims: &T,
        secret: &[u8],
    ) -> Result<String, Self::Error> {
        let header = Header::new(ALGORITHM);
        let token = jsonwebtoken::encode(&header, claims, &EncodingKey::from_secret(secret))
            .map_err(|_| AuthError::AccessTokenSignFailed)?;
        Ok(token)
    }

    #[tracing::instrument]
    fn generate_refresh_token(&self) -> String {
        tempoid::TempoId::generate_custom(::tempoid::TempoIdOptions {
            time_length: 8,
            random_length: 13,
            ..Default::default()
        })
        .to_string()
    }

    #[tracing::instrument]
    fn generate(&self, sub: String) -> Result<TokenIssuerOutput, Self::Error> {
        let now = self.ct.now_utc();
        let iat = now.timestamp();
        let jwt_config = &self.config.jwt;
        let secret = jwt_config.access_token_secret;
        let access_token_period = jwt_config.access_token_period;
        let refresh_token_period = jwt_config.refresh_token_period;
        let access_token_expires_at = now + access_token_period;
        let access_token_expires_at_timestamp = access_token_expires_at.timestamp();
        let refresh_token_expires_at = now + refresh_token_period;
        let claims = UserClaims {
            sub,
            iat,
            exp: access_token_expires_at_timestamp,
        };
        let access_token = self.generate_access_token(&claims, secret)?;
        let refresh_token = self.generate_refresh_token();
        Ok(TokenIssuerOutput {
            access_token,
            refresh_token,
            access_token_expires_at,
            refresh_token_expires_at,
        })
    }

    #[tracing::instrument(skip(access_token, secret))]
    fn verify<T: DeserializeOwned + Clone>(
        &self,
        access_token: &str,
        secret: &[u8],
    ) -> Result<T, Self::Error> {
        let mut validation = Validation::new(ALGORITHM);
        validation.validate_exp = true;
        validation.leeway = 0;
        let token_data =
            jsonwebtoken::decode::<T>(access_token, &DecodingKey::from_secret(secret), &validation)
                .map_err(|_| AuthError::AccessTokenVerifyFailed)?;
        Ok(token_data.claims)
    }

    #[tracing::instrument(skip(access_token))]
    fn decode_without_validation<T: DeserializeOwned + Clone>(
        &self,
        access_token: &str,
    ) -> Result<T, Self::Error> {
        let mut validation = Validation::new(ALGORITHM);
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();
        validation.leeway = 0;
        let tokendata =
            jsonwebtoken::decode(access_token, &DecodingKey::from_secret(&[]), &validation)
                .map_err(|_| AuthError::AccessTokenVerifyFailed)?;
        Ok(tokendata.claims)
    }
}

#[cfg(test)]
mod tests {

    use chrono::Utc;
    use serde::Deserialize;

    use super::*;
    use rstest::*;
    #[fixture]
    async fn token_issuer() -> TokenIssuerImpl {
        TokenIssuerImpl::builder()
            .config(ConfigRef::default())
            .ct(ChronoTz::default())
            .build()
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate_access_token_and_verify_ok(
        #[future(awt)] token_issuer: TokenIssuerImpl,
    ) {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UserClaims {
            sub: String,
            iat: i64,
            exp: i64,
        }
        const SECRET: &[u8] = b"secret";

        let iat = Utc::now().timestamp();
        let exp = iat + 3600;
        let token = token_issuer
            .generate_access_token(
                &UserClaims {
                    sub: "test".to_string(),
                    iat,
                    exp,
                },
                SECRET,
            )
            .unwrap();
        let claims: UserClaims = token_issuer.verify(&token, SECRET).unwrap();
        assert_eq!(claims.sub, "test");
        assert_eq!(claims.iat, iat);
        assert_eq!(claims.exp, exp);
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate_access_token_and_verify_err(
        #[future(awt)] token_issuer: TokenIssuerImpl,
    ) {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UserClaims {
            sub: String,
            iat: i64,
            exp: i64,
        }
        const SECRET: &[u8] = b"secret";
        let iat = Utc::now().timestamp();
        let token1 = token_issuer
            .generate_access_token(
                &UserClaims {
                    sub: "test".to_string(),
                    iat,
                    exp: iat - 3600,
                },
                SECRET,
            )
            .unwrap();
        let token2 = token_issuer
            .generate_access_token(
                &UserClaims {
                    sub: "test".to_string(),
                    iat,
                    exp: iat - 60,
                },
                SECRET,
            )
            .unwrap();
        let result = token_issuer.verify::<UserClaims>(&token1, SECRET);
        assert_eq!(result.err(), Some(AuthError::AccessTokenVerifyFailed));
        let result = token_issuer.verify::<UserClaims>(&token2, SECRET);
        assert_eq!(result.err(), Some(AuthError::AccessTokenVerifyFailed));
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate_refresh_token(#[future(awt)] token_issuer: TokenIssuerImpl) {
        let token = token_issuer.generate_refresh_token();
        assert_eq!(token.len(), 21);
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate(#[future(awt)] token_issuer: TokenIssuerImpl) {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UserClaims {
            sub: String,
            iat: i64,
            exp: i64,
        }
        let token_data = token_issuer.generate("test".to_string()).unwrap();
        let claims: UserClaims = token_issuer
            .decode_without_validation(&token_data.access_token)
            .unwrap();
        assert_eq!(claims.sub, "test");
        assert!(claims.iat > 0);
        assert!(claims.exp > 0);
        assert_eq!(token_data.refresh_token.len(), 21);
    }

    #[rstest]
    #[tokio::test]
    async fn test_decode_without_validation_err(#[future(awt)] token_issuer: TokenIssuerImpl) {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UserClaims {
            sub: String,
            iat: i64,
            exp: i64,
        }
        let token = "xxxx";
        let result = token_issuer.decode_without_validation::<UserClaims>(token);
        assert_eq!(result.err(), Some(AuthError::AccessTokenVerifyFailed));
    }
}
