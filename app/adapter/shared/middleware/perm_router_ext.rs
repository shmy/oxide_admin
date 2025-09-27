use crate::WebState;
use crate::shared::error::WebError;
use crate::shared::extractor::valid_user::ValidUser;
use application::iam::service::iam_service::IamService;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use domain::iam::value_object::permission_group::{PermissionChecker, PermissionGroup};
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouter};

pub trait PermissonRouteExt {
    #[allow(unused)]
    fn permit_all(self, permission_group: PermissionGroup) -> Self;
    #[allow(unused)]
    fn permit_any(self, permission_group: PermissionGroup) -> Self;
}

trait _InternalPermissionExt {
    fn check(self, checker: PermissionChecker) -> Self;
}

impl _InternalPermissionExt for UtoipaMethodRouter<WebState> {
    fn check(mut self, checker: PermissionChecker) -> Self {
        self.2 = self
            .2
            .layer(middleware::from_fn(move |req: Request, next: Next| {
                let checker = checker.clone();
                async move {
                    let (Some(valid_user), Some(iam_service)) = (
                        req.extensions().get::<ValidUser>(),
                        req.extensions().get::<IamService>(),
                    ) else {
                        return Err(WebError::ValidUserNotFound);
                    };
                    iam_service
                        .check_permissions(&valid_user.0, checker)
                        .await?;
                    Ok::<Response, WebError>(next.run(req).await)
                }
            }));
        self
    }
}

impl _InternalPermissionExt for OpenApiRouter<WebState> {
    fn check(mut self, checker: PermissionChecker) -> Self {
        self = self.layer(middleware::from_fn(move |req: Request, next: Next| {
            let checker = checker.clone();
            async move {
                let (Some(valid_user), Some(iam_service)) = (
                    req.extensions().get::<ValidUser>(),
                    req.extensions().get::<IamService>(),
                ) else {
                    return Err(WebError::ValidUserNotFound);
                };
                iam_service
                    .check_permissions(&valid_user.0, checker)
                    .await?;
                Ok::<Response, WebError>(next.run(req).await)
            }
        }));
        self
    }
}

impl PermissonRouteExt for UtoipaMethodRouter<WebState> {
    fn permit_all(self, permission_group: PermissionGroup) -> Self {
        self.check(PermissionChecker::All(permission_group))
    }

    fn permit_any(self, permission_group: PermissionGroup) -> Self {
        self.check(PermissionChecker::Any(permission_group))
    }
}

impl PermissonRouteExt for OpenApiRouter<WebState> {
    fn permit_all(self, permission_group: PermissionGroup) -> Self {
        self.check(PermissionChecker::All(permission_group))
    }

    fn permit_any(self, permission_group: PermissionGroup) -> Self {
        self.check(PermissionChecker::Any(permission_group))
    }
}

#[macro_export]
macro_rules! perms {
    ($($p:expr),*) => {
        domain::iam::value_object::permission_group::PermissionGroup::new(std::collections::HashSet::from([$($p),*]))
    };
}
