use crate::WebState;
use crate::shared::error::WebError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domain::organization::value_object::user_id::UserId;

#[derive(Clone)]
pub struct ValidUser(pub UserId);

impl ValidUser {
    pub fn new(id: UserId) -> Self {
        Self(id)
    }
}

impl FromRequestParts<WebState> for ValidUser {
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &WebState,
    ) -> Result<Self, Self::Rejection> {
        let Some(valid_admin) = parts.extensions.get::<Self>() else {
            return Err(WebError::ValidUserNotFound);
        };
        Ok(valid_admin.clone())
    }
}
