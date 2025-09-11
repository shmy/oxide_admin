mod api;
#[cfg(not(debug_assertions))]
mod frontend;
mod shared;
mod upload;
use axum::Router;
pub use shared::constant::*;
pub use shared::state::*;

pub fn routing(state: WebState) -> Router {
    let router = Router::new()
        .nest("/api", api::routing(state.clone()))
        .with_state(state.clone())
        .merge(upload::routing(state));
    #[cfg(not(debug_assertions))]
    let router = router.merge(frontend::routing());
    router
}
