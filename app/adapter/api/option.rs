use application::{
    auth::service::auth_service::AuthService,
    organization::query::option_roles::OptionRolesQueryHandler,
    shared::{dto::OptionStringDto, query_handler::QueryHandler as _},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    shared::{
        extractor::{accept_language::AcceptLanguage, inject::Inject},
        response::{JsonResponse, JsonResponseType},
        translation::{
            TranslatedMenuTree, TranslatedPermissionTree, tranlate_menus, tranlate_permissions,
        },
    },
};

#[utoipa::path(
    get,
    path = "/role",
    summary = "List roles as option",
    tag = "Option",
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
    path = "/menu",
    summary = "List menu tree",
    tag = "Option",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<TranslatedMenuTree>>))
    )
)]
#[tracing::instrument]
async fn menus(
    language: AcceptLanguage,
    Inject(service): Inject<AuthService>,
) -> JsonResponseType<Vec<TranslatedMenuTree>> {
    let pages = service.get_all_privated_pages();
    let lang_id = language.identifier();
    let menus = tranlate_menus(pages.to_vec(), &lang_id);
    JsonResponse::ok(menus)
}

#[utoipa::path(
    get,
    path = "/permission",
    summary = "List permission tree",
    tag = "Option",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<TranslatedPermissionTree>>))
    )
)]
#[tracing::instrument]
async fn permissions(
    language: AcceptLanguage,
    Inject(service): Inject<AuthService>,
) -> JsonResponseType<Vec<TranslatedPermissionTree>> {
    let tree = service.get_permission_tree();
    let lang_id = language.identifier();
    JsonResponse::ok(tranlate_permissions(tree.to_vec(), lang_id))
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(roles))
        .routes(routes!(menus))
        .routes(routes!(permissions))
}
