use std::net::SocketAddr;

use application::{
    re_export::ChronoTz,
    shared::{bgworker::record_access_log::RecordAccessLog, bgworker_impl::RecordAccessLogStorage},
};
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

    let valid_user = request
        .extensions()
        .get::<ValidUser>()
        .cloned()
        .expect("Failed to get valid user");
    let response = next.run(request).await;
    let status = response.status();
    let elapsed = now.elapsed();
    let ct = state.provider().provide::<ChronoTz>();
    let job = RecordAccessLog::builder()
        .user_id(valid_user.0.to_string())
        .method(method)
        .uri(uri)
        .maybe_user_agent(user_agent)
        .maybe_ip(client_ip)
        .status(status.as_u16() as i16)
        .elapsed(elapsed.as_millis() as i64)
        .occurred_at(ct.now())
        .build();
    if let Err(err) = state
        .provider()
        .provide::<RecordAccessLogStorage>()
        .push(job)
        .await
    {
        tracing::error!(error = %err, "Failed to enqueue record_access_log");
    }
    response
}
