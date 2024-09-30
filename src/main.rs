use axum::Router;
use websocket::handler::ws_routes;

mod websocket;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(ws_routes());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}