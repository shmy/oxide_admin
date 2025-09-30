use application::re_export::provider::Provider;
use axum::{extract::FromRequestParts, http::request::Parts};
use nject::Injectable;

use crate::{WebState, shared::error::WebError};

#[derive(Clone)]
pub struct Inject<T>(pub T);

impl<T> FromRequestParts<WebState> for Inject<T>
where
    for<'prov> T: Injectable<'prov, T, Provider>,
{
    type Rejection = WebError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &WebState,
    ) -> Result<Self, Self::Rejection> {
        let provider = state.provider();
        let instance = provider.provide::<T>();
        Ok(Self(instance))
    }
}
