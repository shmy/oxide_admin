use std::str::FromStr as _;

use application::{re_export::State, shared::bgworker_impl::WorkerRegistry};
use axum::extract::Path;
use domain::auth::value_object::permission::SYSTEM_BGWORKER_READ;
use futures_util::{StreamExt as _, stream};
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
    let namespaces = WorkerRegistry::list_namespaces()
        .iter()
        .map(|ns| {
            (
                ns.name.to_string(),
                ns.concurrency,
                ns.retries,
                ns.timeout.as_secs(),
            )
        })
        .collect::<Vec<_>>();
    let results = stream::iter(namespaces)
        .map(|(name, concurrency, retries, timeout)| async move {
            let stat = WorkerRegistry::stats(&name).await;
            response::Namespace {
                name,
                concurrency,
                retries,
                timeout,
                pending: stat.pending,
                running: stat.running,
                killed: stat.dead,
                failed: stat.failed,
                done: stat.success,
            }
        })
        .buffer_unordered(2)
        .collect()
        .await;
    JsonResponse::ok(results)
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
    let stat = WorkerRegistry::stats(&ns).await;
    JsonResponse::ok(response::Stat {
        pending: stat.pending,
        running: stat.running,
        killed: stat.dead,
        failed: stat.failed,
        done: stat.success,
    })
}

#[utoipa::path(
    get,
    path = "/{ns}/jobs/{state}/{page}",
    summary = "List bgworker jobs by namespace",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<response::JobsResponse>))
    )
)]
#[tracing::instrument]
async fn jobs(
    Path((ns, state, page)): Path<(String, String, i32)>,
) -> JsonResponseType<response::JobsResponse> {
    let Ok(state) = State::from_str(&state) else {
        return JsonResponse::err("invalid state");
    };
    let items = WorkerRegistry::list_jobs(&ns, &state, page).await;
    let items = items
        .into_iter()
        .map(|item| response::Job {
            args: item.args,
            parts: item.parts,
        })
        .collect::<Vec<_>>();
    let len = items.len();
    JsonResponse::ok(response::JobsResponse {
        items,
        has_next: len == 10,
    })
}

mod response {
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct Namespace {
        pub name: String,
        pub concurrency: usize,
        pub retries: usize,
        pub timeout: u64,
        pub pending: usize,
        pub running: usize,
        pub killed: usize,
        pub failed: usize,
        pub done: usize,
    }

    #[derive(Serialize, ToSchema)]
    pub struct Stat {
        pub pending: usize,
        pub running: usize,
        pub killed: usize,
        pub failed: usize,
        pub done: usize,
    }

    #[derive(Serialize, ToSchema)]
    pub struct JobsResponse {
        pub items: Vec<Job>,
        #[serde(rename = "hasNext")]
        pub has_next: bool,
    }

    #[derive(Serialize, ToSchema)]
    pub struct Job {
        pub args: serde_json::Value,
        pub parts: serde_json::Value,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(namespaces).permit_all(perms!(SYSTEM_BGWORKER_READ)))
        .routes(routes!(stat).permit_all(perms!(SYSTEM_BGWORKER_READ)))
        .routes(routes!(jobs).permit_all(perms!(SYSTEM_BGWORKER_READ)))
}
