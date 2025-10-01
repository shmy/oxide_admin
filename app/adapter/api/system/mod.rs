use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod file;
mod option;
mod role;
mod stat;
mod user;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/options", option::routing())
        .nest("/files", file::routing())
        .nest("/stats", stat::routing())
}
