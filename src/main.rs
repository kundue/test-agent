use ping_api::app;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind to port 3000");
    axum::serve(listener, app()).await.expect("serve HTTP requests");
}
