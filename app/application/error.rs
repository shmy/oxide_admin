use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApplicationError>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("不支持的图片格式")]
    UnsupportedImageFormat,
}
