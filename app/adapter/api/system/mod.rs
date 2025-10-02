use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod file;
mod option;
mod role;
mod sched;
mod stat;
mod user;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/users", user::routing())
        .nest("/roles", role::routing())
        .nest("/departments", department::routing())
        .nest("/files", file::routing())
        .nest("/scheds", sched::routing())
        .nest("/options", option::routing())
        .nest("/stats", stat::routing())
}
mod department;
