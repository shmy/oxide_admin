use application::{
    iam::{
        query::option_roles::OptionRolesQueryHandler,
        service::{iam_service::IamService, page::Page},
    },
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

async fn roles(
    Inject(query_handler): Inject<OptionRolesQueryHandler>,
) -> JsonResponseType<Vec<OptionDto>> {
    let items = query_handler.query().await?;
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
