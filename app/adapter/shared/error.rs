use axum::response::{IntoResponse, Response};
use domain::{
    auth::error::AuthError, organization::error::OrganizationError, system::error::SystemError,
};
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error(transparent)]
    Application(#[from] application::error::ApplicationError),
    #[error("invalid_header_value")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
    #[error("authorized_user_not_found")]
    ValidUserNotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Organization(#[from] OrganizationError),
    #[error(transparent)]
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
