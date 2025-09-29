use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod option;
mod profile;
mod role;
mod stat;
mod upload;
mod user;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/profile", profile::routing())
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/options", option::routing())
        .nest("/stats", stat::routing())
        .nest("/uploads", upload::routing())
}
