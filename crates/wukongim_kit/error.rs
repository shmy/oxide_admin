pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] http_client_kit::ReqwestError),
    #[error("{0}")]
    ReqwestMiddleware(#[from] http_client_kit::ReqwestMiddlewareError),
}
