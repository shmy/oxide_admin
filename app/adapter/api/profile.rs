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
    shared::command_handler::CommandHandler,
};

use axum::{
    Json, Router,
    routing::{get, post, put},
};

use crate::{
    WebState,
    shared::{
        extractor::{inject::Inject, valid_user::ValidUser},
        response::{JsonResponse, JsonResponseType},
    },
};

#[tracing::instrument]
async fn sign_out(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<SignOutCommandHandler>,
) -> JsonResponseType<()> {
    let command = SignOutCommand::builder().id(id).build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

#[tracing::instrument]
async fn current(
    ValidUser(id): ValidUser,
    Inject(iam_service): Inject<IamService>,
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
) -> JsonResponseType<response::CurrentResponse> {
    let (mut user, pages) = tokio::try_join!(
        query_handler.query(RetrieveUserQuery::builder().id(id.clone()).build()),
        async { Ok(iam_service.get_available_pages(id).await) }
    )?;
    iam_service
        .replenish_user_portrait(std::slice::from_mut(&mut user))
        .await;
    JsonResponse::ok(response::CurrentResponse { user, pages })
}

#[tracing::instrument(skip(request))]
async fn password(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<UpdateUserSelfPasswordCommandHandler>,
    Json(request): Json<request::UpdateUserPasswordRequest>,
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

    #[derive(Deserialize)]
    pub struct UpdateUserPasswordRequest {
        pub password: String,
        pub new_password: String,
        pub confirm_new_password: String,
    }
}
mod response {
    use application::iam::{dto::user::UserDto, service::page::Page};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct CurrentResponse {
        pub user: UserDto,
        pub pages: [Page; 2],
    }
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route("/sign_out", post(sign_out))
        .route("/current", get(current))
        .route("/password", put(password))
}
