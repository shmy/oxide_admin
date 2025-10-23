use thiserror::Error;

pub type Result<T> = std::result::Result<T, FlagError>;

#[derive(Debug, Error)]
pub enum FlagError {
    #[cfg(feature = "flipt")]
    #[error("{0}")]
    InvalidHeaderValue(#[from] http_client_kit::header::InvalidHeaderValue),

    #[cfg(feature = "flipt")]
    #[error("{0}")]
    ClientError(#[from] flipt::error::ClientError),

    #[cfg(feature = "flipt")]
    #[error("{0}")]
    UrlParse(#[from] url::ParseError),
}
