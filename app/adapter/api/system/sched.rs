use application::{
    shared::{
        command_handler::CommandHandler, paging_result::PagingResult,
        query_handler::QueryHandler as _,
    },
    system::{
        command::batch_delete_scheds::{BatchDeleteSchedsCommand, BatchDeleteSchedsCommandHandler},
        dto::sched::SchedDto,
        query::{
            retrieve_sched::{RetrieveSchedQuery, RetrieveSchedQueryHandler},
            search_scheds::{SearchSchedsQuery, SearchSchedsQueryHandler},
        },
    },
};
use axum::{
    Json,
    extract::{Path, Query},
};
use domain::system::value_object::permission::{SYSTEM_SCHED_DELETE, SYSTEM_SCHED_READ};
use domain::system::value_object::sched_id::SchedId;
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
    path = "/{id}",
    summary = "Retrieve sched",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponse<SchedDto>))
    )
)]
#[tracing::instrument]
async fn retrieve(
    Inject(query_handler): Inject<RetrieveSchedQueryHandler>,
    Path(id): Path<SchedId>,
) -> JsonResponseType<SchedDto> {
    let sched = query_handler
        .query(RetrieveSchedQuery::builder().id(id).build())
        .await?;
    JsonResponse::ok(sched)
}

#[utoipa::path(
    post,
    path = "/batch/delete",
    summary = "Batch delete scheds",
    tag = "System",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteSchedsCommandHandler>,
    Json(command): Json<BatchDeleteSchedsCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_SCHED_READ)))
        .routes(routes!(retrieve).permit_all(perms!(SYSTEM_SCHED_READ)))
        .routes(routes!(batch_delete).permit_all(perms!(SYSTEM_SCHED_DELETE)))
}
