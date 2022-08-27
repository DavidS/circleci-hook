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
    global,
    sdk::export::trace::stdout,
    sdk::trace::Tracer,
    trace::{SpanBuilder, TraceId, Tracer as OtherTracer},
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

    match payload {
        structs::WebhookPayload::PingEvent {
            happened_at,
            id,
            webhook,
        } => {
            state
                .tracer
                .span_builder("test2")
                .with_trace_id(TraceId::from_bytes(*id.as_bytes()))
                .with_start_time(happened_at)
                .with_end_time(happened_at);
        }

        structs::WebhookPayload::JobCompleted {
            happened_at,
            pipeline,
            webhook,
            organization,
            workflow,
            project,
            id,
            job,
        } => {
            if let Some(stopped_at) = job.stopped_at {
                state.tracer.build(
                    SpanBuilder::from_name("job-completed")
                        .with_trace_id(TraceId::from_bytes(*pipeline.id.as_bytes()))
                        .with_start_time(job.started_at)
                        .with_end_time(stopped_at),
                );
            }
        }
        structs::WebhookPayload::WorkflowCompleted {
            id,
            happened_at,
            webhook,
            workflow,
            pipeline,
            project,
            organization,
        } => {
            if let Some(stopped_at) = workflow.stopped_at {
                state.tracer.build(
                    SpanBuilder::from_name("workflow-completed")
                        .with_trace_id(TraceId::from_bytes(*pipeline.id.as_bytes()))
                        .with_start_time(workflow.created_at)
                        .with_end_time(stopped_at),
                );
            }
        }
    }

    "Hello, Mikey and backendsouls!"
}
