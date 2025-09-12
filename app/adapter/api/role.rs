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
    shared::{command_handler::CommandHandler, paging_result::PagingResult},
};
use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::{get, post, put},
};
use domain::iam::value_object::{permission_code::SYSTEM_ROLE, role_id::RoleId};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermRouterExt as _,
        response::{JsonResponse, JsonResponsePagingType, JsonResponseType, PagingResponse},
    },
};

#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchRolesQueryHandler>,
    Query(query): Query<SearchRolesQuery>,
) -> JsonResponsePagingType<RoleDto> {
    let PagingResult { total, items } = query_handler.query_cached(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

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

#[tracing::instrument]
async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteRolesCommandHandler>,
    Json(command): Json<BatchDeleteRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[tracing::instrument]
async fn batch_enable(
    Inject(command_handler): Inject<BatchEnableRolesCommandHandler>,
    Json(command): Json<BatchEnableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[tracing::instrument]
async fn batch_disable(
    Inject(command_handler): Inject<BatchDisableRolesCommandHandler>,
    Json(command): Json<BatchDisableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[tracing::instrument]
async fn create(
    Inject(command_handler): Inject<CreateRoleCommandHandler>,
    Json(command): Json<CreateRoleCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[tracing::instrument]
async fn update(
    Inject(command_handler): Inject<UpdateRoleCommandHandler>,
    Path(_id): Path<RoleId>,
    Json(command): Json<UpdateRoleCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route_with_permission("/", get(search), perms!(SYSTEM_ROLE))
        .route_with_permission("/", post(create), perms!(SYSTEM_ROLE))
        .route_with_permission("/{id}", get(retrieve), perms!(SYSTEM_ROLE))
        .route_with_permission("/{id}", put(update), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/delete", post(batch_delete), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/enable", post(batch_enable), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/disable", post(batch_disable), perms!(SYSTEM_ROLE))
}
