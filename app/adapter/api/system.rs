use application::system::{
    dto::system_snapshot::SystemSnapshot, service::system_service::SystemService,
};
use axum::{Router, routing::get};
use domain::iam::value_object::permission_code::SYSTEM_INFO;

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermRouterExt as _,
        response::{JsonResponse, JsonResponseType},
    },
};

pub async fn system_info(
    Inject(service): Inject<SystemService>,
) -> JsonResponseType<&'static SystemSnapshot> {
    let info = service.info().await?;
    JsonResponse::ok(info)
}

pub fn routing() -> Router<WebState> {
    Router::new().route_with_permission("/info", get(system_info), perms!(SYSTEM_INFO))
}
