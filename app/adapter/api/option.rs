use application::{
    iam::{
        query::option_roles::OptionRolesQueryHandler,
        service::{iam_service::IamService, page::Page},
    },
    shared::{dto::OptionStringDto, query_handler::QueryHandler as _},
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
    path = "/roles",
    summary = "Roles",
    tag = "Options",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<OptionStringDto>>))
    )
)]
#[tracing::instrument]
async fn roles(
    Inject(query_handler): Inject<OptionRolesQueryHandler>,
) -> JsonResponseType<Vec<OptionStringDto>> {
    let items = query_handler.query(()).await?;
    JsonResponse::ok(items)
}

#[utoipa::path(
    get,
    path = "/permissions",
    summary = "Permissions",
    tag = "Options",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<Page>>))
    )
)]
#[tracing::instrument]
async fn permissions(Inject(service): Inject<IamService>) -> JsonResponseType<&'static [Page]> {
    let pages = service.get_all_pages();
    JsonResponse::ok(pages)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(roles))
        .routes(routes!(permissions))
}
