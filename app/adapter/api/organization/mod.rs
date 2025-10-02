use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod department;
mod role;
mod user;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/departments", department::routing())
}
