use axum::response::{IntoResponse, Response};
use domain::{
    auth::error::AuthError, organization::error::OrganizationError, system::error::SystemError,
};
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("{0}")]
    Application(#[from] application::error::ApplicationError),
    #[error("{0}")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
    #[error("Authorized user does not exist")]
    ValidUserNotFound,
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    Organization(#[from] OrganizationError),
    #[error("{0}")]
    System(#[from] SystemError),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let code = self.to_string();
        tracing::error!(error = % self, code);
        let mut response = Response::default();
        response.extensions_mut().insert(WebErrorData { code });
        response
    }
}

#[derive(Clone, Debug)]
pub struct WebErrorData {
    pub code: String,
}
