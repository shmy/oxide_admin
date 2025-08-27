use application::{
    iam::service::{iam_service::IamService, page::Page, role_service::RoleService},
    shared::dto::OptionDto,
};
use axum::{Router, routing::get};

use crate::{
    WebState,
    shared::{
        extractor::inject::Inject,
        response::{JsonResponse, JsonResponseType},
    },
};

async fn roles(Inject(service): Inject<RoleService>) -> JsonResponseType<Vec<OptionDto>> {
    let items = service.get_all().await?;
    JsonResponse::ok(items)
}

async fn permissions(Inject(service): Inject<IamService>) -> JsonResponseType<&'static [Page]> {
    let pages = service.get_all_pages();
    JsonResponse::ok(pages)
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route("/roles", get(roles))
        .route("/permissions", get(permissions))
}
