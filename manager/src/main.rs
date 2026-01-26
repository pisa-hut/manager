mod http;

#[tokio::main]
async fn main() {
    let app = http::router::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
