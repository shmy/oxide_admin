pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("{0}")]
    CborEncode(#[from] minicbor_serde::error::EncodeError<core::convert::Infallible>),

    #[error("{0}")]
    CborDecode(#[from] minicbor_serde::error::DecodeError),
}
