use axum::{
    body::Bytes,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Router,
};
use circleci_hook_app::{
    otel::build_hook_result_span,
    signatures::{parse_signature_header, verify_signature},
    structs::WebhookPayload,
};
use opentelemetry::{
    sdk::{trace as sdktrace, Resource},
    KeyValue,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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
                let payload = serde_json::from_slice::<WebhookPayload>(body.as_ref());
                match payload {
                    Ok(payload) => build_hook_result_span(&payload, &state.tracer),
                    Err(_) => todo!("JSON decode error handling"),
                }
                return "Success!";
            }
        }
    }

    todo!("signature verification failure handling")
}
