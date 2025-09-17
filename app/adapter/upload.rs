use application::{re_export::path::UPLOAD_DIR, system::service::upload_service::UploadService};
use axum::{
    extract::{OriginalUri, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse as _, Response},
    routing::get,
};

use tower::util::ServiceExt as _;
use tower_http::services::ServeDir;
use utoipa_axum::router::OpenApiRouter;

use crate::{UPLOAD_PATH, WebState};

pub fn routing(state: WebState) -> OpenApiRouter {
    let serve_dir = ServeDir::new(UPLOAD_DIR.as_path());
    let router = OpenApiRouter::new()
        .route(
            "/{*path}",
            get(move |req: Request<axum::body::Body>| async move { serve_dir.oneshot(req).await }),
        )
        .layer(middleware::from_fn_with_state(state, limited_middleware));

    let router = OpenApiRouter::new().nest(UPLOAD_PATH, router);
    #[cfg(feature = "trace_otlp")]
    let router = router
        .layer(axum_tracing_opentelemetry::middleware::OtelInResponseLayer)
        .layer(axum_tracing_opentelemetry::middleware::OtelAxumLayer::default());
    router
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
