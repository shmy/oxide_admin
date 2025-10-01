use application::{
    shared::{paging_result::PagingResult, query_handler::QueryHandler as _},
    system::{
        dto::file::FileDto,
        query::search_files::{SearchFilesQuery, SearchFilesQueryHandler},
    },
};
use axum::extract::Query;
use domain::system::value_object::permission::SYSTEM_FILE_READ;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt as _,
        response::{JsonResponse, JsonResponsePagingType, PagingResponse},
    },
};

#[utoipa::path(
    get,
    params(SearchFilesQuery),
    path = "/",
    summary = "Search files",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<FileDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchFilesQueryHandler>,
    Query(query): Query<SearchFilesQuery>,
) -> JsonResponsePagingType<FileDto> {
    let PagingResult { total, items } = query_handler.query(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new().routes(routes!(search).permit_all(perms!(SYSTEM_FILE_READ)))
}
