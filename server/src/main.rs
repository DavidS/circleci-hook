use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize, Debug)]
struct WebHookDescriptor {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct PingEvent {
    happened_at: String,
    id: String,
    #[serde(rename = "type")]
    typë: String,
    webhook: WebHookDescriptor,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(root))
        .route("/", post(hook_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, Mikey and backendsouls!"
}

async fn hook_handler(Json(payload): Json<serde_json::Value>) -> &'static str {
    println!("{:#?}", payload);
    "Hello, Mikey and backendsouls!"
}

// async fn hook_handler(Json(payload): Json<PingEvent>) -> &'static str {
//     println!("{:#?}", payload);
//     "Hello, Mikey and backendsouls!"
// }
