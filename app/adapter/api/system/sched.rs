use application::{
    shared::{
        command_handler::CommandHandler, paging_result::PagingResult,
        query_handler::QueryHandler as _,
    },
    system::{
        command::batch_delete_scheds::{BatchDeleteSchedsCommand, BatchDeleteSchedsCommandHandler},
        dto::sched::{SchedDto, SchedRecordDto},
        query::{
            paging_sched_records::{PagingSchedRecordsQuery, PagingSchedRecordsQueryHandler},
            search_scheds::{SearchSchedsQuery, SearchSchedsQueryHandler},
        },
    },
};
use axum::{Json, extract::Query};
use domain::auth::value_object::permission::{SYSTEM_SCHED_DELETE, SYSTEM_SCHED_READ};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt as _,
        response::{
            JsonResponse, JsonResponseEmpty, JsonResponsePagingType, JsonResponseType,
            PagingResponse,
        },
    },
};

#[utoipa::path(
    get,
    params(SearchSchedsQuery),
    path = "/",
    summary = "Search scheds",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<SchedDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchSchedsQueryHandler>,
    Query(query): Query<SearchSchedsQuery>,
) -> JsonResponsePagingType<SchedDto> {
    let PagingResult { total, items } = query_handler.query(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    get,
    path = "/records",
    summary = "Paging sched records",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<SchedRecordDto>))
    )
)]
#[tracing::instrument]
async fn records(
    Inject(query_handler): Inject<PagingSchedRecordsQueryHandler>,
    Query(query): Query<PagingSchedRecordsQuery>,
) -> JsonResponsePagingType<SchedRecordDto> {
    let PagingResult { total, items } = query_handler.query(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    post,
    path = "/records/batch/delete",
    summary = "Batch delete sched records",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_delete_records(
    Inject(command_handler): Inject<BatchDeleteSchedsCommandHandler>,
    Json(command): Json<BatchDeleteSchedsCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_SCHED_READ)))
        .routes(routes!(records).permit_all(perms!(SYSTEM_SCHED_READ)))
        .routes(routes!(batch_delete_records).permit_all(perms!(SYSTEM_SCHED_DELETE)))
}
