use application::{
    iam::{
        command::{
            batch_delete_users::{BatchDeleteUsersCommand, BatchDeleteUsersCommandHandler},
            batch_disable_users::{BatchDisableUsersCommand, BatchDisableUsersCommandHandler},
            batch_enable_users::{BatchEnableUsersCommand, BatchEnableUsersCommandHandler},
            create_user::{CreateUserCommand, CreateUserCommandHandler},
            update_user::{UpdateUserCommand, UpdateUserCommandHandler},
            update_user_password::{UpdateUserPasswordCommand, UpdateUserPasswordCommandHandler},
        },
        dto::user::UserDto,
        query::{
            retrieve_user::{RetrieveUserQuery, RetrieveUserQueryHandler},
            search_users::{SearchUsersQuery, SearchUsersQueryHandler},
        },
        service::iam_service::IamService,
    },
    shared::{command_handler::CommandHandler, paging_result::PagingResult},
};
use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::{get, post, put},
};
use domain::iam::value_object::{permission_code::SYSTEM_USER, user_id::UserId};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermRouterExt as _,
        response::{JsonResponse, JsonResponsePagingType, JsonResponseType, PagingResponse},
    },
};

async fn search(
    Inject(query_handler): Inject<SearchUsersQueryHandler>,
    Inject(iam_service): Inject<IamService>,
    Query(query): Query<SearchUsersQuery>,
) -> JsonResponsePagingType<UserDto> {
    let PagingResult { total, mut items } = query_handler.query_cached(query).await?;
    iam_service.replenish_user_portrait(&mut items).await;
    JsonResponse::ok(PagingResponse { total, items })
}

async fn retrieve(
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
    Inject(iam_service): Inject<IamService>,
    Path(id): Path<UserId>,
) -> JsonResponseType<UserDto> {
    let mut user = query_handler
        .query(RetrieveUserQuery::builder().id(id).build())
        .await?;
    iam_service
        .replenish_user_portrait(std::slice::from_mut(&mut user))
        .await;
    JsonResponse::ok(user)
}

async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteUsersCommandHandler>,
    Json(command): Json<BatchDeleteUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn batch_enable(
    Inject(command_handler): Inject<BatchEnableUsersCommandHandler>,
    Json(command): Json<BatchEnableUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn batch_disable(
    Inject(command_handler): Inject<BatchDisableUsersCommandHandler>,
    Json(command): Json<BatchDisableUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn create(
    Inject(command_handler): Inject<CreateUserCommandHandler>,
    Json(command): Json<CreateUserCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn update(
    Inject(command_handler): Inject<UpdateUserCommandHandler>,
    Path(_id): Path<UserId>,
    Json(command): Json<UpdateUserCommand>,
) -> JsonResponseType<()> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn update_password(
    Inject(command_handler): Inject<UpdateUserPasswordCommandHandler>,
    Path(id): Path<UserId>,
    Json(request): Json<request::UpdateUserPasswordRequest>,
) -> JsonResponseType<()> {
    let command = UpdateUserPasswordCommand::builder()
        .id(id)
        .new_password(request.new_password)
        .confirm_new_password(request.confirm_new_password)
        .build();
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(())
}

mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct UpdateUserPasswordRequest {
        pub new_password: String,
        pub confirm_new_password: String,
    }
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route_with_permission("/", get(search), perms!(SYSTEM_USER))
        .route_with_permission("/", post(create), perms!(SYSTEM_USER))
        .route_with_permission("/{id}", get(retrieve), perms!(SYSTEM_USER))
        .route_with_permission("/{id}", put(update), perms!(SYSTEM_USER))
        .route_with_permission("/batch/delete", post(batch_delete), perms!(SYSTEM_USER))
        .route_with_permission("/batch/enable", post(batch_enable), perms!(SYSTEM_USER))
        .route_with_permission("/batch/disable", post(batch_disable), perms!(SYSTEM_USER))
        .route_with_permission("/{id}/password", put(update_password), perms!(SYSTEM_USER))
}
