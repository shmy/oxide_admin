use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod option;
mod role;
mod stat;
mod user;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/options", option::routing())
        .nest("/stats", stat::routing())
}
