use std::time::Duration;

use application::{
    iam::command::{
        refresh_token::{RefreshTokenCommand, RefreshTokenCommandHandler},
        sign_in::{SignInCommand, SignInCommandHandler},
        sign_out::{SignOutCommand, SignOutCommandHandler},
    },
    shared::command_handler::CommandHandler,
};
use axum::{Json, Router, routing::post};

use crate::{
    WebState,
    shared::{
        extractor::inject::Inject,
        middleware::rate_limit_ext::RateLimitRouterExt,
        response::{JsonResponse, JsonResponseType},
    },
};

async fn sign_in(
    Inject(command_handler): Inject<SignInCommandHandler>,
    Json(command): Json<SignInCommand>,
) -> JsonResponseType<response::SignInResponse> {
    let output = command_handler.handle(command).await?;
    JsonResponse::ok(response::SignInResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
    })
}

async fn sign_out(
    Inject(command_handler): Inject<SignOutCommandHandler>,
    Json(command): Json<SignOutCommand>,
) -> JsonResponseType<()> {
    command_handler.handle(command).await?;
    JsonResponse::ok(())
}

async fn refresh_token(
    Inject(command_handler): Inject<RefreshTokenCommandHandler>,
    Json(command): Json<RefreshTokenCommand>,
) -> JsonResponseType<response::SignInResponse> {
    let output = command_handler.handle(command).await?;
    JsonResponse::ok(response::SignInResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
    })
}

mod response {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct SignInResponse {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route("/sign_in", post(sign_in))
        .route("/sign_out", post(sign_out))
        .route("/refresh_token", post(refresh_token))
        // 每秒最多1次
        .rate_limit_layer(Duration::from_secs(1), 1)
}
