use anyhow::Result;

pub mod math;

pub struct CaptchaData {
    pub bytes: Vec<u8>,
    pub value: String,
}

pub trait CaptchaTrait {
    fn generate(&self) -> Result<CaptchaData>;
}
