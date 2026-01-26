mod app_state;
mod db;
mod http;
mod migrator;

use crate::app_state::AppState;

const DATABASE_URL: &str = "sqlite://db.sqlite?mode=rwc";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = db::connect(DATABASE_URL).await;
    db::migrate(&db).await.unwrap();

    let state = AppState { db };

    let app = http::router::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
