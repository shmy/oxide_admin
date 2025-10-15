use application::{
    organization::{
        command::{
            batch_delete_departments::{
                BatchDeleteDepartmentsCommand, BatchDeleteDepartmentsCommandHandler,
            },
            create_department::{CreateDepartmentCommand, CreateDepartmentCommandHandler},
            update_department::{UpdateDepartmentCommand, UpdateDepartmentCommandHandler},
        },
        dto::department::DepartmentWithChildren,
        query::tree_departments::{TreeDepartmentsQuery, TreeDepartmentsQueryHandler},
    },
    shared::{command_handler::CommandHandler, query_handler::QueryHandler as _},
};
use axum::{
    Json,
    extract::{Path, Query},
};
use domain::{
    auth::value_object::permission::{
        ORGANIZATION_DEPARTMENT_CREATE, ORGANIZATION_DEPARTMENT_DELETE,
        ORGANIZATION_DEPARTMENT_READ, ORGANIZATION_DEPARTMENT_UPDATE,
    },
    organization::value_object::department_id::DepartmentId,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt as _,
        response::{JsonResponse, JsonResponseEmpty, JsonResponseType},
    },
};

#[utoipa::path(
    get,
    params(TreeDepartmentsQuery),
    path = "/",
    summary = "Tree departments",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponse<Vec<DepartmentWithChildren>>))
    )
)]
#[tracing::instrument]
async fn tree(
    Inject(query_handler): Inject<TreeDepartmentsQueryHandler>,
    Query(query): Query<TreeDepartmentsQuery>,
) -> JsonResponseType<Vec<DepartmentWithChildren>> {
    let items = query_handler.query(query).await?;
    JsonResponse::ok(items)
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
        .routes(routes!(tree).permit_all(perms!(ORGANIZATION_DEPARTMENT_READ)))
        .routes(routes!(create).permit_all(perms!(ORGANIZATION_DEPARTMENT_CREATE)))
        .routes(routes!(update).permit_all(perms!(ORGANIZATION_DEPARTMENT_UPDATE)))
        .routes(routes!(batch_delete).permit_all(perms!(ORGANIZATION_DEPARTMENT_DELETE)))
}
