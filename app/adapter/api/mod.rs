use axum::Router;

use crate::{WebState, shared::middleware::user_authn_required::user_authn_required};

mod auth;
mod option;
mod profile;
mod role;
mod system;
mod upload;
mod user;

pub fn routing(state: WebState) -> Router<WebState> {
    Router::new()
        .nest("/profile", profile::routing())
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/options", option::routing())
        .nest("/systems", system::routing())
        .nest("/uploads", upload::routing())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            user_authn_required,
        ))
        .nest("/auth", auth::routing())
        .layer(axum_tracing_opentelemetry::middleware::OtelInResponseLayer)
        .layer(axum_tracing_opentelemetry::middleware::OtelAxumLayer::default())
}
