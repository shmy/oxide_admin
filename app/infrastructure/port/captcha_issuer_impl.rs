use captcha_generator::CaptchaTrait as _;
use domain::{
    iam::error::IamError,
    shared::{
        id_generator::IdGenerator,
        port::captcha_issuer::{Captcha, CaptchaIssuerTrait},
    },
};
use nject::injectable;

use crate::shared::kv::{Kv, KvTrait as _};

#[injectable]
pub struct CaptchaIssuerImpl {
    kv: Kv,
}

impl CaptchaIssuerImpl {
    fn fill_captcha_key(key: &str) -> String {
        format!("captcha:{key}")
    }
}
impl CaptchaIssuerTrait for CaptchaIssuerImpl {
    type Error = IamError;
    async fn generate_with_ttl(&self, ttl: std::time::Duration) -> Result<Captcha, Self::Error> {
        let math = captcha_generator::math::MathCaptcha::new(100, 140, 40);
        let captcha_data = math
            .generate()
            .map_err(|_| IamError::CaptchaFailedGenerate)?;
        let key = IdGenerator::random();
        let full_key = Self::fill_captcha_key(&key);
        self.kv
            .set_with_ex(&full_key, captcha_data.value, ttl)
            .map_err(|_| IamError::CaptchaFailedGenerate)?;
        Ok(Captcha {
            bytes: captcha_data.bytes,
            key,
        })
    }

    async fn verify(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        let full_key = Self::fill_captcha_key(key);
        let existing_value = self
            .kv
            .get::<String>(&full_key)
            .map_err(|_| IamError::CaptchaInvalid)?;
        if existing_value != value {
            return Err(IamError::CaptchaIncorrect);
        }

        let _ = self.kv.delete(&full_key);
        Ok(())
    }
}
