use application::{
    organization::{
        command::{
            batch_delete_departments::{
                BatchDeleteDepartmentsCommand, BatchDeleteDepartmentsCommandHandler,
            },
            create_department::{CreateDepartmentCommand, CreateDepartmentCommandHandler},
            update_department::{UpdateDepartmentCommand, UpdateDepartmentCommandHandler},
        },
        dto::department::DepartmentDto,
        query::{
            retrieve_department::{RetrieveDepartmentQuery, RetrieveDepartmentQueryHandler},
            search_departments::{SearchDepartmentsQuery, SearchDepartmentsQueryHandler},
        },
    },
    shared::{
        command_handler::CommandHandler, paging_result::PagingResult,
        query_handler::QueryHandler as _,
    },
};
use axum::{
    Json,
    extract::{Path, Query},
};
use domain::{
    organization::value_object::department_id::DepartmentId,
    system::value_object::permission::{
        SYSTEM_DEPARTMENT_CREATE, SYSTEM_DEPARTMENT_DELETE, SYSTEM_DEPARTMENT_READ,
        SYSTEM_DEPARTMENT_UPDATE,
    },
};
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
    params(SearchDepartmentsQuery),
    path = "/",
    summary = "Search departments",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<DepartmentDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchDepartmentsQueryHandler>,
    Query(query): Query<SearchDepartmentsQuery>,
) -> JsonResponsePagingType<DepartmentDto> {
    let PagingResult { total, items } = query_handler.query(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    get,
    path = "/{id}",
    summary = "Retrieve department",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponse<DepartmentDto>))
    )
)]
#[tracing::instrument]
async fn retrieve(
    Inject(query_handler): Inject<RetrieveDepartmentQueryHandler>,
    Path(id): Path<DepartmentId>,
) -> JsonResponseType<DepartmentDto> {
    let department = query_handler
        .query(RetrieveDepartmentQuery::builder().id(id).build())
        .await?;
    JsonResponse::ok(department)
}

#[utoipa::path(
    post,
    path = "/batch/delete",
    summary = "Batch delete departments",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteDepartmentsCommandHandler>,
    Json(command): Json<BatchDeleteDepartmentsCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/",
    summary = "Create department",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn create(
    Inject(command_handler): Inject<CreateDepartmentCommandHandler>,
    Json(command): Json<CreateDepartmentCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    put,
    path = "/{id}",
    summary = "Update department",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn update(
    Inject(command_handler): Inject<UpdateDepartmentCommandHandler>,
    Path(_id): Path<DepartmentId>,
    Json(command): Json<UpdateDepartmentCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_DEPARTMENT_READ)))
        .routes(routes!(retrieve).permit_all(perms!(SYSTEM_DEPARTMENT_READ)))
        .routes(routes!(create).permit_all(perms!(SYSTEM_DEPARTMENT_CREATE)))
        .routes(routes!(update).permit_all(perms!(SYSTEM_DEPARTMENT_UPDATE)))
        .routes(routes!(batch_delete).permit_all(perms!(SYSTEM_DEPARTMENT_DELETE)))
}
