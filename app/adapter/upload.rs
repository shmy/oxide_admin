use application::{re_export::path::UPLOAD_DIR, system::service::upload_service::UploadService};
use axum::{
    Router,
    extract::{OriginalUri, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse as _, Response},
};

use tower_http::services::ServeDir;

use crate::{UPLOAD_PATH, WebState};

pub fn routing(state: WebState) -> Router {
    let serve_dir = ServeDir::new(UPLOAD_DIR.as_path());
    let router = Router::new()
        .fallback_service(serve_dir)
        .layer(middleware::from_fn_with_state(state, limited_middleware));
    Router::new().nest(UPLOAD_PATH, router)
}

async fn limited_middleware(
    State(state): State<WebState>,
    uri: OriginalUri,
    request: Request,
    next: Next,
) -> Response {
    let service = state.provider().provide::<UploadService>();
    let verified = service.verify_url(uri.0);
    if !verified {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}
