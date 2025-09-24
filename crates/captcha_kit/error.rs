use thiserror::Error;

pub type Result<T> = std::result::Result<T, CaptchaError>;

#[derive(Debug, Error)]
pub enum CaptchaError {
    #[error("{0}")]
    ImageError(#[from] image::ImageError),
}
