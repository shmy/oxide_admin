use std::time::Duration;

use application::{
    auth::command::{
        refresh_captcha::{RefreshCaptchaCommand, RefreshCaptchaCommandHandler},
        refresh_token::{RefreshTokenCommand, RefreshTokenCommandHandler},
        sign_in::{SignInCommand, SignInCommandHandler},
    },
    shared::command_handler::CommandHandler,
};
use axum::{
    Json,
    http::{
        HeaderName, HeaderValue,
        header::{self, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    WebState,
    shared::{
        error::WebError,
        extractor::inject::Inject,
        middleware::rate_limit_ext::RateLimitRouterExt as _,
        response::{JsonResponse, JsonResponseType},
    },
};

const CAPTCHA_HEADER_NAME: HeaderName = HeaderName::from_static("x-captcha-id");
const CAPTCHA_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("image/png");

#[utoipa::path(
    post,
    path = "/sign_in",
    summary = "Sign in",
    tag = "Auth",
    responses(
        (status = 200, body = inline(JsonResponse<response::SignInResponse>))
    )
)]
#[tracing::instrument]
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

#[utoipa::path(
    post,
    path = "/token",
    summary = "Refresh Token",
    tag = "Auth",
    responses(
        (status = 200, body = inline(JsonResponse<response::SignInResponse>))
    )
)]
#[tracing::instrument]
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

#[utoipa::path(
    get,
    path = "/captcha",
    summary = "Refresh captcha",
    tag = "Auth",
    responses(
        (status = 200, body = inline(Vec<u8>))
    )
)]
#[tracing::instrument]
async fn refresh_captcha(
    Inject(command_handler): Inject<RefreshCaptchaCommandHandler>,
) -> Result<impl IntoResponse, WebError> {
    let data = command_handler
        .handle(RefreshCaptchaCommand::builder().build())
        .await?;
    let captcha_header_value = HeaderValue::from_str(&data.key)?;
    let mut headers = header::HeaderMap::new();
    headers.insert(CONTENT_TYPE, CAPTCHA_CONTENT_TYPE);
    headers.insert(CAPTCHA_HEADER_NAME, captcha_header_value);
    Ok((headers, data.bytes))
}

mod response {
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct SignInResponse {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(sign_in).rate_limit_layer(Duration::from_secs(3), 1))
        .routes(routes!(refresh_token).rate_limit_layer(Duration::from_secs(5), 1))
        .routes(routes!(refresh_captcha))
}
