use crate::WebState;
use crate::shared::extractor::valid_user::ValidUser;
use crate::shared::middleware::common::{
    get_access_token_from_header, get_access_token_from_query, unauthorized,
};
use application::iam::service::iam_service::IamService;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn user_authn_required(
    State(state): State<WebState>,
    header_map: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let maybe_token = get_access_token_from_header(&header_map)
        .or_else(|| get_access_token_from_query(request.uri()));
    let Some(access_token) = maybe_token else {
        return unauthorized("请提供 Token");
    };
    let service = state.provider().provide::<IamService>();
    let id = match service.verify_token(&access_token).await {
        Ok(admin_id) => admin_id,
        Err(err) => {
            return unauthorized(err.to_string());
        }
    };

    let extensions_mut = request.extensions_mut();
    extensions_mut.insert::<ValidUser>(ValidUser::new(id));
    extensions_mut.insert::<IamService>(service);
    next.run(request).await
}
