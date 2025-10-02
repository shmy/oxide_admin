use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod file;
mod option;
mod sched;
mod stat;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/files", file::routing())
        .nest("/scheds", sched::routing())
        .nest("/options", option::routing())
        .nest("/stats", stat::routing())
}
