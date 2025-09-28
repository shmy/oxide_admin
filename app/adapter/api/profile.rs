use application::{
    iam::{
        command::{
            sign_out::{SignOutCommand, SignOutCommandHandler},
            update_user_self_password::{
                UpdateUserSelfPasswordCommand, UpdateUserSelfPasswordCommandHandler,
            },
        },
        query::retrieve_user::{RetrieveUserQuery, RetrieveUserQueryHandler},
        service::iam_service::IamService,
    },
    shared::{command_handler::CommandHandler, query_handler::QueryHandler as _},
};

use axum::Json;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    shared::{
        extractor::{inject::Inject, valid_user::ValidUser},
        response::{JsonResponse, JsonResponseEmpty, JsonResponseType},
    },
};

#[utoipa::path(
    post,
    path = "/sign_out",
    summary = "Sign out",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument]
async fn sign_out(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<SignOutCommandHandler>,
) -> JsonResponseType<()> {
    let command = SignOutCommand::builder().id(id).build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[utoipa::path(
    get,
    path = "/current",
    summary = "Current user",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponse<response::CurrentResponse>))
    )
)]
#[tracing::instrument]
async fn current(
    ValidUser(id): ValidUser,
    Inject(iam_service): Inject<IamService>,
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
) -> JsonResponseType<response::CurrentResponse> {
    let (mut user, pages, permissions) = tokio::try_join!(
        query_handler.query(RetrieveUserQuery::builder().id(id.clone()).build()),
        async { Ok(iam_service.get_available_pages(&id).await) },
        async { Ok(iam_service.get_available_permissions(&id).await) }
    )?;
    iam_service
        .replenish_user_portrait(std::slice::from_mut(&mut user))
        .await;
    JsonResponse::ok(response::CurrentResponse {
        user,
        pages,
        permissions,
    })
}

#[utoipa::path(
    post,
    path = "/password",
    summary = "Change self password",
    tag = "Profile",
    responses(
        (status = 200, body = inline(JsonResponseEmpty))
    )
)]
#[tracing::instrument(skip(request))]
async fn password(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<UpdateUserSelfPasswordCommandHandler>,
    Json(request): Json<request::UpdatePasswordRequest>,
) -> JsonResponseType<()> {
    let command = UpdateUserSelfPasswordCommand::builder()
        .id(id)
        .password(request.password)
        .new_password(request.new_password)
        .confirm_new_password(request.confirm_new_password)
        .build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

mod request {
    use serde::Deserialize;
    use utoipa::ToSchema;

    #[derive(Deserialize, ToSchema)]
    pub struct UpdatePasswordRequest {
        pub password: String,
        pub new_password: String,
        pub confirm_new_password: String,
    }
}
mod response {
    use application::iam::{dto::user::UserDto, service::menu::MenuTree};
    use domain::iam::value_object::permission::Permission;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct CurrentResponse {
        pub user: UserDto,
        pub pages: [MenuTree; 2],
        pub permissions: Vec<&'static Permission>,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(current))
        .routes(routes!(sign_out))
        .routes(routes!(password))
}
