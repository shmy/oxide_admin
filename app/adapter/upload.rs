use application::re_export::{
    hmac_util::{HmacUtil, UPLOAD_PATH},
    path::UPLOAD_DIR,
};
use axum::{
    Router,
    extract::{OriginalUri, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse as _, Response},
};

use tower_http::services::ServeDir;

use crate::WebState;

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
    let hmac_util = state.provider().provide::<HmacUtil>();
    let verified = hmac_util.verify_path(uri.0);
    if !verified {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}
