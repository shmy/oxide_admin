use crate::WebState;
use crate::shared::error::WebError;
use crate::shared::extractor::valid_user::ValidUser;
use application::iam::service::iam_service::IamService;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::MethodRouter;
use axum::{Router, middleware};
use domain::iam::value_object::permission_group::PermissionGroup;

pub trait PermRouterExt {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<WebState>,
        permission_group: PermissionGroup,
    ) -> Self;
}

impl PermRouterExt for Router<WebState> {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<WebState>,
        permission_group: PermissionGroup,
    ) -> Self {
        self.route(
            path,
            method_router.route_layer(middleware::from_fn(move |req: Request, next: Next| {
                let permission_group = permission_group.clone();
                async move {
                    let (Some(valid_user), Some(iam_service)) = (
                        req.extensions().get::<ValidUser>(),
                        req.extensions().get::<IamService>(),
                    ) else {
                        return Err(anyhow::anyhow!("未找到 ValidUser").into());
                    };
                    iam_service
                        .check_permission(&valid_user.0, &permission_group)
                        .await?;
                    Ok::<Response, WebError>(next.run(req).await)
                }
            })),
        )
    }
}

#[macro_export]
macro_rules! perms {
    ($($p:expr),*) => {
        domain::iam::value_object::permission_group::PermissionGroup::new(std::collections::HashSet::from([$($p),*]))
    };
}
