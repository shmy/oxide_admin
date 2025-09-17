mod api;
#[cfg(not(debug_assertions))]
mod frontend;
mod shared;
mod upload;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
pub use shared::constant::*;
pub use shared::state::*;
use utoipa::OpenApi as _;
use utoipa_axum::router::OpenApiRouter;

#[derive(utoipa::OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(title = "Oxide Admin API", description = "This is a API for Oxide Admin<br/> Hint: Please replace paging with page and page_size in search* apis. issue: https://github.com/juhaku/utoipa/issues/841"),
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

pub fn routing(state: WebState, with_openapi: bool) -> Router {
    let (mut router, open_api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::routing(state.clone()))
        .with_state(state.clone())
        .merge(upload::routing(state))
        .route("/health", get(health))
        .split_for_parts();
    if with_openapi {
        router = {
            use utoipa_scalar::Servable as _;
            router.merge(utoipa_scalar::Scalar::with_url("/scalar", open_api))
        };
    }
    #[cfg(not(debug_assertions))]
    let router = router.merge(frontend::routing());
    router
}
async fn health() -> StatusCode {
    StatusCode::OK
}
