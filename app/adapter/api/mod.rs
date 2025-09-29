use utoipa_axum::router::OpenApiRouter;

use crate::{
    WebState,
    shared::middleware::{api_error::api_error, user_authn_required::user_authn_required},
};

mod authn;
mod profile;
mod system;

pub fn routing(state: WebState) -> OpenApiRouter<WebState> {
    let router = OpenApiRouter::new()
        .nest("/system", system::routing())
        .nest("/profile", profile::routing())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            user_authn_required,
        ))
        .nest("/authn", authn::routing());
    #[cfg(feature = "trace_otlp")]
    let router = router
        .layer(axum_tracing_opentelemetry::middleware::OtelInResponseLayer)
        .layer(axum_tracing_opentelemetry::middleware::OtelAxumLayer::default());
    router.layer(axum::middleware::from_fn(api_error))
}
