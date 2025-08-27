use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::response::JsonResponse;

pub struct WebError(anyhow::Error);

impl<E> From<E> for WebError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let info = self.0.to_string();
        tracing::error!(error = %self.0, info);
        (StatusCode::OK, JsonResponse::<()>::err(info)).into_response()
    }
}
