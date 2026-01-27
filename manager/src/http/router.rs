use axum::{Router, routing::get};

use crate::{app_state::AppState, http::handlers};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health))
        .route("/avs", get(handlers::av::list_avs))
        .with_state(state)
}
