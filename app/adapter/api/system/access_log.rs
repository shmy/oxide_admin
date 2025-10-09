use application::{
    shared::{paging_result::PagingResult, query_handler::QueryHandler as _},
    system::{
        dto::access_log::AccessLogDto,
        query::{
            retrieve_access_log::{RetrieveAccessLogQuery, RetrieveAccessLogQueryHandler},
            search_access_logs::{SearchAccessLogsQuery, SearchAccessLogsQueryHandler},
        },
    },
};
use axum::extract::{Path, Query};
use domain::auth::value_object::permission::SYSTEM_ACCESS_LOG_READ;
use domain::system::value_object::access_log_id::AccessLogId;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt as _,
        response::{JsonResponse, JsonResponsePagingType, JsonResponseType, PagingResponse},
    },
};

#[utoipa::path(
    get,
    params(SearchAccessLogsQuery),
    path = "/",
    summary = "Search access_logs",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<AccessLogDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchAccessLogsQueryHandler>,
    Query(query): Query<SearchAccessLogsQuery>,
) -> JsonResponsePagingType<AccessLogDto> {
    let PagingResult { total, items } = query_handler.query(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    get,
    path = "/{id}",
    summary = "Retrieve access_log",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<AccessLogDto>))
    )
)]
#[tracing::instrument]
async fn retrieve(
    Inject(query_handler): Inject<RetrieveAccessLogQueryHandler>,
    Path(id): Path<AccessLogId>,
) -> JsonResponseType<AccessLogDto> {
    let access_log = query_handler
        .query(RetrieveAccessLogQuery::builder().id(id).build())
        .await?;
    JsonResponse::ok(access_log)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_ACCESS_LOG_READ)))
        .routes(routes!(retrieve).permit_all(perms!(SYSTEM_ACCESS_LOG_READ)))
}
