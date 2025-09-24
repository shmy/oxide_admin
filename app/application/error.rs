use thiserror::Error;

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("不支持的图片格式")]
    UnsupportedImageFormat,
}
