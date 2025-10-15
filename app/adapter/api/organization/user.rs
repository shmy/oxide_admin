use application::{
    auth::service::auth_service::AuthService,
    organization::{
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
use domain::auth::value_object::permission::{
    ORGANIZATION_USER_CREATE, ORGANIZATION_USER_DELETE, ORGANIZATION_USER_DISABLE,
    ORGANIZATION_USER_ENABLE, ORGANIZATION_USER_READ, ORGANIZATION_USER_UPDATE,
    ORGANIZATION_USER_UPDATE_PASSWORD,
};
use domain::organization::value_object::user_id::UserId;
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
    params(SearchUsersQuery),
    path = "/",
    summary = "Search users",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponse<PagingResponse<UserDto>>))
    )
)]
#[tracing::instrument]
async fn search(
    Inject(query_handler): Inject<SearchUsersQueryHandler>,
    Inject(service): Inject<AuthService>,
    Query(query): Query<SearchUsersQuery>,
) -> JsonResponsePagingType<UserDto> {
    let PagingResult { total, mut items } = query_handler.cached_query(query).await?;
    service.replenish_user_portrait(&mut items).await;
    JsonResponse::ok(PagingResponse { total, items })
}

#[utoipa::path(
    get,
    path = "/{id}",
    summary = "Retrieve user",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponse<UserDto>))
    )
)]
#[tracing::instrument]
async fn retrieve(
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
    Inject(service): Inject<AuthService>,
    Path(id): Path<UserId>,
) -> JsonResponseType<UserDto> {
    let mut user = query_handler
        .query(RetrieveUserQuery::builder().id(id).build())
        .await?;
    service
        .replenish_user_portrait(std::slice::from_mut(&mut user))
        .await;
    JsonResponse::ok(user)
}

#[utoipa::path(
    post,
    path = "/batch/delete",
    summary = "Batch delete users",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_delete(
    Inject(command_handler): Inject<BatchDeleteUsersCommandHandler>,
    Json(command): Json<BatchDeleteUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/batch/enable",
    summary = "Batch enable users",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_enable(
    Inject(command_handler): Inject<BatchEnableUsersCommandHandler>,
    Json(command): Json<BatchEnableUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/batch/disable",
    summary = "Batch disable users",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn batch_disable(
    Inject(command_handler): Inject<BatchDisableUsersCommandHandler>,
    Json(command): Json<BatchDisableUsersCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    post,
    path = "/",
    summary = "Create user",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn create(
    Inject(command_handler): Inject<CreateUserCommandHandler>,
    Json(command): Json<CreateUserCommand>,
) -> JsonResponseType<UserId> {
    let user = command_handler.handle(command).await?;
    JsonResponse::ok(user.id.clone())
}

#[utoipa::path(
    put,
    path = "/{id}",
    summary = "Update user",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn update(
    Inject(command_handler): Inject<UpdateUserCommandHandler>,
    Path(id): Path<UserId>,
    Json(command): Json<UpdateUserCommand>,
) -> JsonResponseType<UserId> {
    let _ = command_handler.handle(command).await?;
    JsonResponse::ok(id)
}

#[utoipa::path(
    put,
    path = "/{id}/password",
    summary = "Update user password",
    tag = "Organization",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument(skip(request))]
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
    use utoipa::ToSchema;

    #[derive(Deserialize, ToSchema)]
    pub struct UpdateUserPasswordRequest {
        pub new_password: String,
        pub confirm_new_password: String,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(search).permit_all(perms!(ORGANIZATION_USER_READ)))
        .routes(routes!(retrieve).permit_all(perms!(ORGANIZATION_USER_READ)))
        .routes(routes!(batch_delete).permit_all(perms!(ORGANIZATION_USER_DELETE)))
        .routes(routes!(batch_enable).permit_all(perms!(ORGANIZATION_USER_ENABLE)))
        .routes(routes!(batch_disable).permit_all(perms!(ORGANIZATION_USER_DISABLE)))
        .routes(routes!(create).permit_all(perms!(ORGANIZATION_USER_CREATE)))
        .routes(routes!(update).permit_all(perms!(ORGANIZATION_USER_UPDATE)))
        .routes(routes!(update_password).permit_all(perms!(ORGANIZATION_USER_UPDATE_PASSWORD)))
}
