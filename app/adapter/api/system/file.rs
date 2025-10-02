use application::{
    shared::{paging_result::PagingResult, query_handler::QueryHandler as _},
    system::{
        dto::file::FileDto,
        query::search_files::{SearchFilesQuery, SearchFilesQueryHandler},
        service::upload_service::UploadService,
    },
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use domain::auth::value_object::permission::{SYSTEM_FILE_DOWNLOAD, SYSTEM_FILE_READ};
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

#[utoipa::path(
    get,
    path = "/download/{*path}",
    summary = "Download file",
    tag = "System",
    responses(
        (status = 200)
    )
)]
#[tracing::instrument]
async fn download(
    Inject(service): Inject<UploadService>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if let Ok(url) = service.presign_url(path).await {
        return Redirect::temporary(&url).into_response();
    }
    StatusCode::BAD_REQUEST.into_response()
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_FILE_READ)))
        .routes(routes!(download).permit_all(perms!(SYSTEM_FILE_DOWNLOAD)))
}
