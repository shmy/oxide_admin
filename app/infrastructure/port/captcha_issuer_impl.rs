use captcha_generator::CaptchaTrait as _;
use domain::{
    iam::error::IamError,
    shared::{
        id_generator::IdGenerator,
        port::captcha_issuer::{Captcha, CaptchaIssuerTrait},
    },
};
use kvdb_kit::{Kvdb, KvdbTrait as _};
use nject::injectable;

#[derive(Debug)]
#[injectable]
pub struct CaptchaIssuerImpl {
    kvdb: Kvdb,
}

impl CaptchaIssuerImpl {
    fn fill_captcha_key(key: &str) -> String {
        format!("captcha:{key}")
    }
}
impl CaptchaIssuerTrait for CaptchaIssuerImpl {
    type Error = IamError;
    #[tracing::instrument]
    async fn generate_with_ttl(&self, ttl: std::time::Duration) -> Result<Captcha, Self::Error> {
        let math = captcha_generator::math::MathCaptcha::new(100, 140, 40);
        let captcha_data = math
            .generate()
            .map_err(|_| IamError::CaptchaGenerationFailed)?;
        let key = IdGenerator::random();
        let full_key = Self::fill_captcha_key(&key);
        self.kvdb
            .set_with_ex(&full_key, captcha_data.value, ttl)
            .await
            .map_err(|_| IamError::CaptchaGenerationFailed)?;
        Ok(Captcha {
            bytes: captcha_data.bytes,
            key,
        })
    }

    #[tracing::instrument]
    async fn verify(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        let full_key = Self::fill_captcha_key(key);
        let Some(existing_value) = self.kvdb.get::<String>(&full_key).await else {
            return Err(IamError::CaptchaInvalid);
        };

        if existing_value != value {
            return Err(IamError::CaptchaIncorrect);
        }

        let _ = self.kvdb.delete(&full_key).await;
        Ok(())
    }
}
