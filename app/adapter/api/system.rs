use application::system::{
    dto::system_snapshot::SystemSnapshot, service::system_service::SystemService,
};
use domain::iam::value_object::permission_code::SYSTEM_INFO;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt,
        response::{JsonResponse, JsonResponseType},
    },
};

#[utoipa::path(
    get,
    path = "/info",
    summary = "System info",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<SystemSnapshot>))
    )
)]
#[tracing::instrument]
pub async fn system_info(
    Inject(service): Inject<SystemService>,
) -> JsonResponseType<&'static SystemSnapshot> {
    let info = service.info().await?;
    JsonResponse::ok(info)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new().routes(routes!(system_info).permit_all(perms!(SYSTEM_INFO)))
}
