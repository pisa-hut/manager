use axum::{
    Router,
    routing::{get, post},
};

use crate::{app_state::AppState, http::handlers};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health))
        .route("/av", get(handlers::av::list_av))
        .route("/av", post(handlers::av::create_av))
        .with_state(state)
}
