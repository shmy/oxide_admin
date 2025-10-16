use axum::{Json, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    shared::{
        extractor::accept_language::LANGUAGE_COOKIE_NAME,
        middleware::{
            access_log::access_log, api_error::api_error, user_authn_required::user_authn_required,
        },
        response::{JsonResponse, JsonResponseEmpty},
    },
};

mod auth;
mod option;
mod organization;
mod profile;
mod system;
mod upload;

pub fn routing(state: WebState) -> OpenApiRouter<WebState> {
    let router = OpenApiRouter::new()
        .nest("/profile", profile::routing())
        .nest("/organization", organization::routing())
        .nest("/system", system::routing())
        .nest("/uploads", upload::routing())
        .nest("/options", option::routing())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            access_log,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            user_authn_required,
        ))
        .nest("/auth", auth::routing())
        .routes(routes!(set_language));
    #[cfg(feature = "trace_otlp")]
    let router = router
        .layer(axum_tracing_opentelemetry::middleware::OtelInResponseLayer)
        .layer(axum_tracing_opentelemetry::middleware::OtelAxumLayer::default());
    router.layer(axum::middleware::from_fn(api_error))
}

#[utoipa::path(
    post,
    path = "/language",
    summary = "Set current language",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument(skip(request))]
async fn set_language(
    cookie_jar: CookieJar,
    Json(request): Json<SetLanguageRequest>,
) -> impl IntoResponse {
    let language_cookie = Cookie::build((LANGUAGE_COOKIE_NAME, request.lang_id))
        .path("/")
        .secure(true)
        .http_only(true)
        .build();
    let set_cookie = cookie_jar.add(language_cookie);
    (set_cookie, JsonResponse::ok(())).into_response()
}

#[derive(Deserialize, ToSchema)]
pub struct SetLanguageRequest {
    pub lang_id: String,
}
