use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use tokio::time::Instant;

use crate::{WebState, shared::extractor::valid_user::ValidUser};

pub async fn access_log(State(state): State<WebState>, request: Request, next: Next) -> Response {
    let now = Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let client_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string());

    let valid_user = request.extensions().get::<ValidUser>().cloned();
    let response = next.run(request).await;
    let status = response.status();
    let elapsed = now.elapsed();
    tracing::info!(
        method = method,
        uri = uri,
        user_agent = user_agent,
        client_ip = client_ip,
        status = status.as_u16(),
        valid_user = valid_user.map(|v| v.0.to_string()),
        elapsed = elapsed.as_millis(),
        "Request handled",
    );
    response
}
