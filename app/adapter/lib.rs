mod api;
mod frontend;
mod shared;
mod upload;
use axum::Router;
pub use shared::state::*;

pub fn routing(state: WebState) -> Router {
    Router::new()
        .nest("/api", api::routing(state.clone()))
        .with_state(state.clone())
        .merge(frontend::routing())
        .merge(upload::routing(state))
}
