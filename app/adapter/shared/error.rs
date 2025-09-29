use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::system::error::IamError;

use super::response::JsonResponse;

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("{0}")]
    Application(#[from] application::error::ApplicationError),

    #[error("{0}")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),

    #[error("授权用户不存在")]
    ValidUserNotFound,

    #[error("{0}")]
    IamError(#[from] IamError),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let info = self.to_string();
        tracing::error!(error = %self, info);
        (StatusCode::OK, JsonResponse::<()>::err(info)).into_response()
    }
}
