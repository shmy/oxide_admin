use domain::{
    iam::error::IamError,
    shared::token_issuer::{TokenIssuerOutput, TokenIssuerTrait, UserClaims},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use nject::injectable;
use serde::{Serialize, de::DeserializeOwned};

use crate::shared::{chrono_tz::ChronoTz, config::Config};

const ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Clone)]
#[injectable]
pub struct TokenIssuerImpl {
    config: Config,
    tz: ChronoTz,
}

impl TokenIssuerTrait for TokenIssuerImpl {
    type Error = IamError;

    fn generate_access_token<T: Serialize>(
        &self,
        claims: &T,
        secret: &[u8],
    ) -> anyhow::Result<String, Self::Error> {
        let header = Header::new(ALGORITHM);
        let token = jsonwebtoken::encode(&header, claims, &EncodingKey::from_secret(secret))
            .map_err(|_| IamError::AccessTokenFailedGenerate)?;
        Ok(token)
    }

    fn generate_refresh_token(&self) -> String {
        tempoid::TempoId::generate_custom(::tempoid::TempoIdOptions {
            time_length: 8,
            random_length: 13,
            ..Default::default()
        })
        .to_string()
    }

    fn generate(&self, sub: String) -> Result<TokenIssuerOutput, Self::Error> {
        let now = self.tz.now_utc();
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
        let access_token = self
            .generate_access_token(&claims, secret)
            .map_err(|_| IamError::AccessTokenFailedGenerate)?;
        let refresh_token = self.generate_refresh_token();
        Ok(TokenIssuerOutput {
            access_token,
            refresh_token,
            access_token_expires_at,
            refresh_token_expires_at,
        })
    }

    fn verify<T: DeserializeOwned>(
        &self,
        access_token: &str,
        secret: &[u8],
    ) -> Result<T, Self::Error> {
        let mut validation = Validation::new(ALGORITHM);
        validation.validate_exp = true;
        validation.leeway = 0;
        let token_data =
            jsonwebtoken::decode::<T>(access_token, &DecodingKey::from_secret(secret), &validation)
                .map_err(|_| IamError::AccessTokenFailedVerify)?;
        Ok(token_data.claims)
    }

    fn decode_without_validation<T: DeserializeOwned>(
        &self,
        access_token: &str,
    ) -> Result<T, Self::Error> {
        let mut validation = Validation::new(ALGORITHM);
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();
        validation.leeway = 0;
        let tokendata =
            jsonwebtoken::decode(access_token, &DecodingKey::from_secret(&[]), &validation)
                .map_err(|_| IamError::AccessTokenFailedVerify)?;
        Ok(tokendata.claims)
    }
}
