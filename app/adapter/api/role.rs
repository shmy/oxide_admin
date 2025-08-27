use application::{
    iam::{
        command::{
            batch_delete_role::{BatchDeleteRolesCommand, BatchDeleteRolesCommandHandler},
            batch_disable_roles::{BatchDisableRolesCommand, BatchDisableRolesCommandHandler},
            batch_enable_roles::{BatchEnableRolesCommand, BatchEnableRolesCommandHandler},
            create_role::{CreateRoleCommand, CreateRoleCommandHandler},
            update_role::{UpdateRoleCommand, UpdateRoleCommandHandler},
        },
        dto::role::RoleDto,
        service::role_service::{RoleService, SearchRolesQuery},
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

async fn paging(
    Inject(service): Inject<RoleService>,
    Query(query): Query<SearchRolesQuery>,
) -> JsonResponsePagingType<RoleDto> {
    let PagingResult { total, items } = service.search_cached(query).await?;
    JsonResponse::ok(PagingResponse { total, items })
}

async fn retrieve(
    Inject(service): Inject<RoleService>,
    Path(id): Path<RoleId>,
) -> JsonResponseType<RoleDto> {
    let role = service.retrieve(id).await?;
    JsonResponse::ok(role)
}

async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteRolesCommandHandler>,
    Json(command): Json<BatchDeleteRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn batch_enable(
    Inject(command_handler): Inject<BatchEnableRolesCommandHandler>,
    Json(command): Json<BatchEnableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn batch_disable(
    Inject(command_handler): Inject<BatchDisableRolesCommandHandler>,
    Json(command): Json<BatchDisableRolesCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn create(
    Inject(command_handler): Inject<CreateRoleCommandHandler>,
    Json(command): Json<CreateRoleCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

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
        .route_with_permission("/", get(paging), perms!(SYSTEM_ROLE))
        .route_with_permission("/", post(create), perms!(SYSTEM_ROLE))
        .route_with_permission("/{id}", get(retrieve), perms!(SYSTEM_ROLE))
        .route_with_permission("/{id}", put(update), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/delete", post(batch_delete), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/enable", post(batch_enable), perms!(SYSTEM_ROLE))
        .route_with_permission("/batch/disable", post(batch_disable), perms!(SYSTEM_ROLE))
}
