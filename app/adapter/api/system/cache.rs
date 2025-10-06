use application::system::service::system_service::{CacheTreeItem, SystemService};
use axum::Json;
use domain::auth::value_object::permission::{SYSTEM_CACHE_DELETE, SYSTEM_CACHE_READ};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt,
        response::{JsonResponse, JsonResponseEmpty, JsonResponseType},
    },
};

#[utoipa::path(
    get,
    path = "/",
    summary = "Tree cache",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<CacheTreeItem>>))
    )
)]
#[tracing::instrument]
async fn tree(Inject(service): Inject<SystemService>) -> JsonResponseType<Vec<CacheTreeItem>> {
    let items = service.cache().await?;

    JsonResponse::ok(items)
}

#[utoipa::path(
    post,
    path = "/delete",
    summary = "Delete cache by prefix",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn delete(
    Inject(service): Inject<SystemService>,
    Json(request): Json<request::DeleteCacheByPrefixRequest>,
) -> JsonResponseType<()> {
    service.delete_cache_by_prefix(&request.prefix).await?;
    JsonResponse::ok(())
}

mod request {
    use serde::Deserialize;
    use utoipa::ToSchema;

    #[derive(Debug, Deserialize, ToSchema)]
    pub struct DeleteCacheByPrefixRequest {
        pub prefix: String,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(tree).permit_all(perms!(SYSTEM_CACHE_READ)))
        .routes(routes!(delete).permit_all(perms!(SYSTEM_CACHE_DELETE)))
}
