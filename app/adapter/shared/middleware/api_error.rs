use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    middleware::Next,
    response::Response,
};

use crate::shared::response::JsonResponse;
const JSON_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/json");

pub async fn api_error(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    let is_json_content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .map(|value| value == JSON_CONTENT_TYPE)
        .unwrap_or(false);

    let status = response.status();
    if !status.is_success() && !is_json_content_type {
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        let data: JsonResponse<()> = JsonResponse::builder()
            .status(1)
            .msg(String::from_utf8_lossy(&body_bytes).to_string())
            .build();
        let new_body = Body::from(serde_json::to_vec(&data).expect("Json encode"));

        let mut new_response = Response::new(new_body);
        *new_response.status_mut() = StatusCode::OK;
        new_response
            .headers_mut()
            .insert(CONTENT_TYPE, JSON_CONTENT_TYPE);

        return new_response;
    }

    response
}
