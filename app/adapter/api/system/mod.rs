use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

mod access_log;
mod bgworker;
mod cache;
mod file;
mod sched;
mod stat;

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .nest("/files", file::routing())
        .nest("/scheds", sched::routing())
        .nest("/stats", stat::routing())
        .nest("/bgworkers", bgworker::routing())
        .nest("/caches", cache::routing())
        .nest("/access_logs", access_log::routing())
}
