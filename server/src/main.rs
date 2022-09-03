use arrayref::array_ref;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use circleci_hook_app::signatures::{parse_signature_header, verify_signature};
use opentelemetry::{
    global,
    sdk::export::trace::stdout,
    sdk::{trace as sdktrace, Resource},
    trace::{
        SpanBuilder, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer as OtherTracer,
    },
    Context, KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;
// use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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
            opentelemetry_otlp::new_exporter().tonic(), // .with_endpoint(env!("OTEL_EXPORTER_OTLP_ENDPOINT"))
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
        .route("/", post(hook_handler))
        .layer(
            ServiceBuilder::new()
                // .layer(middleware::from_fn(validate_circleci_signature))
                .layer(TraceLayer::new_for_http()),
        );

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

fn trace_id_from_pipeline_id(pipeline_id: &Uuid) -> TraceId {
    TraceId::from_bytes(*pipeline_id.as_bytes())
}

fn workflow_span_id_from_pipeline_id(pipeline_id: &Uuid) -> SpanId {
    SpanId::from_bytes(*array_ref!(pipeline_id.as_bytes(), 0, 8))
}

fn create_workflow_context(pipeline_id: Uuid) -> Context {
    let cx = Context::current();
    return cx.with_remote_span_context(SpanContext::new(
        trace_id_from_pipeline_id(&pipeline_id),
        workflow_span_id_from_pipeline_id(&pipeline_id),
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    ));
}

#[cfg(test)]
mod cx_tests {
    use opentelemetry::trace::TraceContextExt;
    use uuid::Uuid;

    use super::create_workflow_context;

    #[test]
    fn test_has_active_span() {
        assert!(create_workflow_context(Uuid::new_v4()).has_active_span());
    }
}

async fn hook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> &'static str {
    println!("Received request");
    if let Some(header_value) = headers
        .get("circleci-signature")
        .and_then(|header| header.to_str().ok())
    {
        if let Some(signature_hex) = parse_signature_header(header_value) {
            if verify_signature(body.as_ref(), b"FOOBAR", signature_hex) {
                let payload = serde_json::from_slice::<structs::WebhookPayload>(body.as_ref());
                match payload {
                    Ok(payload) => match payload {
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
                                println!("Processing JobCompleted");
                                // TODO: try to wedge in the parent span_id from the workflow. Apparently this would require a Context that holds the actual parent span. This sounds too complicated for now. See https://github.com/open-telemetry/opentelemetry-rust/blob/043e4b7523f66e79338ada84e7ab2da53251d448/opentelemetry-api/src/trace/context.rs#L261-L266
                                let cx = create_workflow_context(pipeline.id);
                                // println!("{:#?}", cx.span());
                                // println!("{:#?}", cx.span().span_context());
                                state.tracer.build_with_context(
                                    SpanBuilder::from_name("job")
                                        .with_span_id(SpanId::from_bytes(*array_ref!(
                                            job.id.as_bytes(),
                                            0,
                                            8
                                        )))
                                        .with_start_time(job.started_at)
                                        .with_end_time(stopped_at),
                                    &cx,
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
                                println!("Processing WorkflowCompleted");
                                state.tracer.build(
                                    SpanBuilder::from_name("workflow")
                                        .with_trace_id(trace_id_from_pipeline_id(&pipeline.id))
                                        .with_span_id(workflow_span_id_from_pipeline_id(
                                            &pipeline.id,
                                        ))
                                        .with_start_time(workflow.created_at)
                                        .with_end_time(stopped_at),
                                );
                            }
                        }
                    },
                    Err(_) => todo!("JSON decode error handling"),
                }
                return "Success!";
            }
        }
    }

    todo!("signature verification failure handling")
}
