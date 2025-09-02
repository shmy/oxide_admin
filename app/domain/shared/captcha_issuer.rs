use std::time::Duration;

pub struct Captcha {
    pub bytes: Vec<u8>,
    pub key: String,
}

pub trait CaptchaIssuerTrait {
    type Error;
    fn generate_with_ttl(
        &self,
        ttl: Duration,
    ) -> impl Future<Output = Result<Captcha, Self::Error>>;
    fn verify(&self, key: &str, value: &str) -> impl Future<Output = Result<(), Self::Error>>;
}
