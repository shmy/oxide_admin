use application::{
    iam::{
        command::{
            batch_delete_roles::{BatchDeleteRolesCommand, BatchDeleteRolesCommandHandler},
            batch_disable_roles::{BatchDisableRolesCommand, BatchDisableRolesCommandHandler},
            batch_enable_roles::{BatchEnableRolesCommand, BatchEnableRolesCommandHandler},
            create_role::{CreateRoleCommand, CreateRoleCommandHandler},
            update_role::{UpdateRoleCommand, UpdateRoleCommandHandler},
        },
        dto::role::RoleDto,
        query::{
            retrieve_role::{RetrieveRoleQuery, RetrieveRoleQueryHandler},
            search_roles::{SearchRolesQuery, SearchRolesQueryHandler},
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
use domain::iam::value_object::{
    permission::{
        SYSTEM_ROLE_CREATE, SYSTEM_ROLE_DELETE, SYSTEM_ROLE_DISABLE, SYSTEM_ROLE_ENABLE,
        SYSTEM_ROLE_READ, SYSTEM_ROLE_UPDATE,
    },
    role_id::RoleId,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt,
        response::{
            JsonResponse, JsonResponseEmpty, JsonResponsePagingType, JsonResponseType,
            PagingResponse,
        },
    },
};

#[utoipa::path(
    get,
    params(SearchRolesQuery),
    path = "/",
    summary = "Search roles",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<RoleDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchRolesQueryHandler>,
    Query(query): Query<SearchRolesQuery>,
) -> JsonResponsePagingType<RoleDto> {
    let PagingResult { total, items } = query_handler.query_cached(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    get,
    path = "/{id}",
    summary = "Retrieve role",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponse<RoleDto>))
    )
)]
#[tracing::instrument]
async fn retrieve(
    Inject(query_handler): Inject<RetrieveRoleQueryHandler>,
    Path(id): Path<RoleId>,
) -> JsonResponseType<RoleDto> {
    let role = query_handler
        .query(RetrieveRoleQuery::builder().id(id).build())
        .await?;
    JsonResponse::ok(role)
}

#[utoipa::path(
    post,
    path = "/batch/delete",
    summary = "Batch delete roles",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteRolesCommandHandler>,
    Json(command): Json<BatchDeleteRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/batch/enable",
    summary = "Batch enable roles",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_enable(
    Inject(command_handler): Inject<BatchEnableRolesCommandHandler>,
    Json(command): Json<BatchEnableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/batch/disable",
    summary = "Batch disable roles",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_disable(
    Inject(command_handler): Inject<BatchDisableRolesCommandHandler>,
    Json(command): Json<BatchDisableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/",
    summary = "Create role",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn create(
    Inject(command_handler): Inject<CreateRoleCommandHandler>,
    Json(command): Json<CreateRoleCommand>,
) -> JsonResponseType<RoleId> {
    let role = command_handler.handle(command).await?;
    JsonResponse::ok(role.id.clone())
}

#[utoipa::path(
    put,
    path = "/{id}",
    summary = "Update role",
    tag = "Iam",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn update(
    Inject(command_handler): Inject<UpdateRoleCommandHandler>,
    Path(id): Path<RoleId>,
    Json(command): Json<UpdateRoleCommand>,
) -> JsonResponseType<RoleId> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(id)
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(SYSTEM_ROLE_READ)))
        .routes(routes!(retrieve).permit_all(perms!(SYSTEM_ROLE_READ)))
        .routes(routes!(batch_delete).permit_all(perms!(SYSTEM_ROLE_DELETE)))
        .routes(routes!(batch_enable).permit_all(perms!(SYSTEM_ROLE_ENABLE)))
        .routes(routes!(batch_disable).permit_all(perms!(SYSTEM_ROLE_DISABLE)))
        .routes(routes!(create).permit_all(perms!(SYSTEM_ROLE_CREATE)))
        .routes(routes!(update).permit_all(perms!(SYSTEM_ROLE_UPDATE)))
}
