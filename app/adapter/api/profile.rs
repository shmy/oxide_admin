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
    re_export::hmac_util::HmacUtil,
    shared::command_handler::CommandHandler,
};

use axum::{
    Json, Router,
    routing::{get, post, put},
};

use crate::{
    WebState,
    api::user::sign_user_portrait,
    shared::{
        extractor::{inject::Inject, valid_user::ValidUser},
        response::{JsonResponse, JsonResponseType},
    },
};

async fn sign_out(
    ValidUser(id): ValidUser,
    Inject(command_handler): Inject<SignOutCommandHandler>,
) -> JsonResponseType<()> {
    let command = SignOutCommand::builder().id(id).build();
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn current(
    ValidUser(id): ValidUser,
    Inject(service): Inject<IamService>,
    Inject(hmac_util): Inject<HmacUtil>,
    Inject(query_handler): Inject<RetrieveUserQueryHandler>,
) -> JsonResponseType<response::CurrentResponse> {
    let (user, pages) = tokio::try_join!(
        query_handler.query(RetrieveUserQuery::builder().id(id.clone()).build()),
        async { Ok(service.get_available_pages(id).await) }
    )?;
    let user = sign_user_portrait(user, &hmac_util);
    JsonResponse::ok(response::CurrentResponse { user, pages })
}

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
