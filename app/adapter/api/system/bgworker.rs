use application::shared::bgworker_impl::RecordAccessLogImpl;
use domain::auth::value_object::permission::SYSTEM_SCHED_READ;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        middleware::perm_router_ext::PermissonRouteExt as _,
        response::{JsonResponse, JsonResponseType},
    },
};

#[utoipa::path(
    get,
    path = "/",
    summary = "Stat bgworkers",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<response::Stat>))
    )
)]
#[tracing::instrument]
async fn stat() -> JsonResponseType<response::Stat> {
    let stat = RecordAccessLogImpl::stats().await;
    JsonResponse::ok(response::Stat {
        pending: stat.pending,
        running: stat.running,
        dead: stat.dead,
        failed: stat.failed,
        success: stat.success,
    })
}

mod response {
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct Stat {
        pub pending: usize,
        pub running: usize,
        pub dead: usize,
        pub failed: usize,
        pub success: usize,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new().routes(routes!(stat).permit_all(perms!(SYSTEM_SCHED_READ)))
}
