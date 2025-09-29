use application::{
    shared::{dto::OptionStringDto, query_handler::QueryHandler as _},
    system::{query::option_roles::OptionRolesQueryHandler, service::iam_service::IamService},
};
use domain::system::value_object::{menu::MenuTree, permission::PermissionTree};
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
    path = "/menus",
    summary = "menus",
    tag = "Options",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<MenuTree>>))
    )
)]
#[tracing::instrument]
async fn menus(Inject(service): Inject<IamService>) -> JsonResponseType<&'static [MenuTree]> {
    let pages = service.get_all_privated_pages();
    JsonResponse::ok(pages)
}

#[utoipa::path(
    get,
    path = "/permissions",
    summary = "permissions",
    tag = "Options",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<PermissionTree>>))
    )
)]
#[tracing::instrument]
async fn permissions(
    Inject(service): Inject<IamService>,
) -> JsonResponseType<&'static [PermissionTree]> {
    let tree = service.get_permission_tree();
    JsonResponse::ok(tree)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(roles))
        .routes(routes!(menus))
        .routes(routes!(permissions))
}
