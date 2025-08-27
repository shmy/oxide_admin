use axum::Json;
use serde::Serialize;

use super::error::WebError;

pub type JsonResponseType<T> = anyhow::Result<Json<JsonResponse<T>>, WebError>;
pub type JsonResponsePagingType<T> =
    anyhow::Result<Json<JsonResponse<PagingResponse<T>>>, WebError>;

#[derive(Debug, Serialize)]
pub struct PagingResponse<T> {
    pub total: i64,
    pub items: Vec<T>,
}

#[derive(Debug, Serialize)]
pub struct JsonResponse<T> {
    status: u8,
    msg: String,
    data: Option<T>,
}

impl<T> JsonResponse<T> {
    pub fn ok(data: T) -> JsonResponseType<T> {
        Ok(Json(JsonResponse {
            status: 0,
            msg: "OK".to_string(),
            data: Some(data),
        }))
    }

    pub fn err(info: impl AsRef<str>) -> JsonResponseType<T> {
        Ok(Json(JsonResponse {
            status: 1,
            msg: info.as_ref().to_string(),
            data: None,
        }))
    }
}
