use axum::{
    Router,
    routing::{get, post},
};

use crate::http::handlers;

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health))
        .route("/task/lease", post(handlers::task::lease_task))
        .route("/task/complete", post(handlers::task::complete_task))
}
