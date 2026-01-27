mod app_state;
mod db;
mod entity;
mod http;
mod migrator;

use crate::app_state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let db = db::connect(&std::env::var("DATABASE_URL").expect("DATABSE_URL")).await;
    db::migrate(&db).await.unwrap();

    let state = AppState { db };

    let app = http::router::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
