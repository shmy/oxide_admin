use application::shared::bgworker_impl::WorkerRegistry;
use axum::extract::Path;
use domain::auth::value_object::permission::SYSTEM_BGWORKER_READ;
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
    summary = "List bgworker namespaces",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<response::Namespace>>))
    )
)]
#[tracing::instrument]
async fn namespaces() -> JsonResponseType<Vec<response::Namespace>> {
    let namespaces = WorkerRegistry::list_namespaces();
    JsonResponse::ok(
        namespaces
            .iter()
            .map(|ns| response::Namespace { name: ns.name })
            .collect(),
    )
}

#[utoipa::path(
    get,
    path = "/{ns}/stat",
    summary = "Stat bgworker by namespace",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<response::Stat>))
    )
)]
#[tracing::instrument]
async fn stat(Path(ns): Path<String>) -> JsonResponseType<response::Stat> {
    let stat = WorkerRegistry::stats(ns).await;
    JsonResponse::ok(response::Stat {
        pending: stat.pending,
        running: stat.running,
        dead: stat.dead,
        failed: stat.failed,
        success: stat.success,
    })
}

#[utoipa::path(
    get,
    path = "/{ns}/workers",
    summary = "List wokers by namespace",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<response::Worker>>))
    )
)]
#[tracing::instrument]
async fn workers(Path(ns): Path<String>) -> JsonResponseType<Vec<response::Worker>> {
    let workers = WorkerRegistry::list_workers(ns).await;
    JsonResponse::ok(
        workers
            .into_iter()
            .map(|w| response::Worker {
                id: w.id().to_string(),
                source: w.source.to_string(),
                r#type: w.r#type.to_string(),
            })
            .collect(),
    )
}

mod response {
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct Namespace {
        pub name: &'static str,
    }

    #[derive(Serialize, ToSchema)]
    pub struct Worker {
        pub id: String,
        pub source: String,
        pub r#type: String,
    }

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
    OpenApiRouter::new()
        .routes(routes!(namespaces).permit_all(perms!(SYSTEM_BGWORKER_READ)))
        .routes(routes!(stat).permit_all(perms!(SYSTEM_BGWORKER_READ)))
        .routes(routes!(workers).permit_all(perms!(SYSTEM_BGWORKER_READ)))
}
