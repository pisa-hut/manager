mod app_state;
mod db;
mod http;
mod migrator;

use dotenv::dotenv;
use std::env;

use crate::app_state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let db = db::connect(&env::var("DATABASE_URL").unwrap()).await;
    db::migrate(&db).await.unwrap();

    let state = AppState { db };

    let app = http::router::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
