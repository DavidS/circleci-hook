use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use opentelemetry::{
    global, sdk::export::trace::stdout, sdk::trace::Tracer, trace::Tracer as OtherTracer,
};

mod structs;

#[derive(Clone)]
struct AppState {
    tracer: Tracer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let tracer = stdout::new_pipeline().install_simple();
    let state = AppState { tracer };

    let app = Router::with_state(state)
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

// async fn hook_handler(Json(payload): Json<serde_json::Value>) -> &'static str {
//     println!("{:#?}", payload);
//     "Hello, Mikey and backendsouls!"
// }

async fn hook_handler(
    State(state): State<AppState>,
    Json(payload): Json<structs::WebhookPayload>,
) -> &'static str {
    state.tracer.in_span("test", |cx| {
        println!("{:#?}", payload);
    });
    "Hello, Mikey and backendsouls!"
}
