use std::time::Duration;

use axum::{Router, body::Body, http::Response, response::IntoResponse as _};
use tower_governor::{GovernorError, GovernorLayer, governor::GovernorConfigBuilder};

use crate::{WebState, shared::response::JsonResponse};

pub trait RateLimitRouterExt {
    #[allow(unused)]
    fn rate_limit_route_layer(self, period: Duration, burst_size: u32) -> Self;
    #[allow(unused)]
    fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self;
}

impl RateLimitRouterExt for Router<WebState> {
    fn rate_limit_route_layer(self, period: Duration, burst_size: u32) -> Self {
        let governor_conf = GovernorConfigBuilder::default()
            .period(period)
            .burst_size(burst_size)
            .finish()
            .expect("build governor configuration");
        self.route_layer(GovernorLayer::new(governor_conf).error_handler(error_handler))
    }

    fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self {
        let governor_conf = GovernorConfigBuilder::default()
            .period(period)
            .burst_size(burst_size)
            .finish()
            .expect("build governor configuration");
        self.layer(GovernorLayer::new(governor_conf).error_handler(error_handler))
    }
}

fn error_handler(err: GovernorError) -> Response<Body> {
    match err {
        GovernorError::TooManyRequests { wait_time, .. } => {
            JsonResponse::<()>::err(format!("请求过快，请等待{wait_time}秒"))
        }
        GovernorError::UnableToExtractKey => JsonResponse::<()>::err("无法提取限流密钥"),
        GovernorError::Other { code, msg, .. } => {
            JsonResponse::<()>::err(format!("限流错误：{code}: {msg:?}"))
        }
    }
    .into_response()
}
