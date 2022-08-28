use arrayref::array_ref;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use opentelemetry::{
    global,
    sdk::export::trace::stdout,
    sdk::{trace as sdktrace, Resource},
    trace::{SpanBuilder, SpanId, TraceId, Tracer as OtherTracer},
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
// use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};

mod structs;

#[derive(Clone)]
struct AppState {
    tracer: sdktrace::Tracer,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // let mut map = MetadataMap::new();
    // map.insert(
    //     "x-honeycomb-team",
    //     env!("HONEYCOMB_API_KEY").parse().unwrap(),
    // );

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                // .with_endpoint(env!("OTEL_EXPORTER_OTLP_ENDPOINT"))
                // .with_env()
                // .with_tls_config(ClientTlsConfig::new())
                // .with_metadata(map),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "circleci-hook",
            )])),
        )
        .install_simple()
        // .install_batch(opentelemetry::runtime::Tokio)
        .expect("build an OTLP tracer");
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
    println!("Received request");
    match payload {
        structs::WebhookPayload::PingEvent {
            happened_at,
            id,
            webhook,
        } => {
            state.tracer.build(
                SpanBuilder::from_name("ping")
                    .with_trace_id(TraceId::from_bytes(*id.as_bytes()))
                    .with_start_time(happened_at)
                    .with_end_time(happened_at),
            );
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
                // TODO: try to wedge in the parent span_id from the workflow. Apparently this would require a Context that holds the actual parent span. This sounds too complicated for now. See https://github.com/open-telemetry/opentelemetry-rust/blob/043e4b7523f66e79338ada84e7ab2da53251d448/opentelemetry-api/src/trace/context.rs#L261-L266
                state.tracer.build(
                    SpanBuilder::from_name("job")
                        .with_trace_id(TraceId::from_bytes(*pipeline.id.as_bytes()))
                        .with_span_id(SpanId::from_bytes(*array_ref!(job.id.as_bytes(), 0, 8)))
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
                    SpanBuilder::from_name("workflow")
                        .with_trace_id(TraceId::from_bytes(*pipeline.id.as_bytes()))
                        .with_span_id(SpanId::from_bytes(*array_ref!(
                            pipeline.id.as_bytes(),
                            0,
                            8
                        )))
                        .with_start_time(workflow.created_at)
                        .with_end_time(stopped_at),
                );
            }
        }
    }

    "Success!"
}
