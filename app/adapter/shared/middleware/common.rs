use axum::{
    http::{HeaderMap, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::shared::response::JsonResponse;

pub(crate) fn unauthorized(msg: impl AsRef<str>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        JsonResponse::<()>::err(msg.as_ref()),
    )
        .into_response()
}
pub(crate) fn get_access_token_from_header(header_map: &HeaderMap) -> Option<String> {
    header_map.get(header::AUTHORIZATION).and_then(|value| {
        value.to_str().ok().and_then(|auth_str| {
            let mut parts = auth_str.splitn(2, ' ');
            match (parts.next(), parts.next()) {
                (Some(scheme), Some(token)) if scheme.eq_ignore_ascii_case("bearer") => {
                    Some(token.to_string())
                }
                _ => None,
            }
        })
    })
}

pub(crate) fn get_access_token_from_query(uri: &Uri) -> Option<String> {
    uri.query()
        .and_then(|query| serde_urlencoded::from_str::<AccessTokenInQuery>(query).ok())
        .map(|query| query.access_token)
}

#[derive(Deserialize)]
pub(crate) struct AccessTokenInQuery {
    access_token: String,
}
