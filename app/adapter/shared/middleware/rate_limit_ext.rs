use std::time::Duration;

use axum::{body::Body, http::Response, response::IntoResponse as _};
use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorError, GovernorLayer, governor::GovernorConfigBuilder,
    key_extractor::PeerIpKeyExtractor,
};
use utoipa_axum::router::{UtoipaMethodRouter, UtoipaMethodRouterExt as _};

use crate::{WebState, shared::response::JsonResponse};
type AxumGovernorLayer = GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware, Body>;

fn build_governor_layer(period: Duration, burst_size: u32) -> AxumGovernorLayer {
    let cfg = GovernorConfigBuilder::default()
        .period(period)
        .burst_size(burst_size)
        .finish()
        .expect("build governor configuration");
    GovernorLayer::new(cfg).error_handler(error_handler)
}

pub trait RateLimitRouterExt {
    fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self;
}

// impl RateLimitRouterExt for MethodRouter<WebState> {
//     fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self {
//         self.layer(build_governor_layer(period, burst_size))
//     }
// }

// impl RateLimitRouterExt for Router<WebState> {
//     fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self {
//         self.layer(build_governor_layer(period, burst_size))
//     }
// }

impl RateLimitRouterExt for UtoipaMethodRouter<WebState> {
    fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self {
        self.layer(build_governor_layer(period, burst_size))
    }
}

// impl RateLimitRouterExt for OpenApiRouter<WebState> {
//     fn rate_limit_layer(self, period: Duration, burst_size: u32) -> Self {
//         self.layer(build_governor_layer(period, burst_size))
//     }
// }

fn error_handler(err: GovernorError) -> Response<Body> {
    JsonResponse::<()>::err(err.to_string()).into_response()
}
