use application::system::{
    dto::system_snapshot::SystemSnapshot, service::system_service::SystemService,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    shared::{
        extractor::inject::Inject,
        response::{JsonResponse, JsonResponseType},
    },
};

#[utoipa::path(
    get,
    path = "/info",
    summary = "System stat",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<SystemSnapshot>))
    )
)]
#[tracing::instrument]
pub async fn system_stat(
    Inject(service): Inject<SystemService>,
) -> JsonResponseType<&'static SystemSnapshot> {
    let info = service.info().await?;
    JsonResponse::ok(info)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new().routes(routes!(system_stat))
}
