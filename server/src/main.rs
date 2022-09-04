use axum::{
    body::Bytes,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Router,
};
use circleci_hook_app::{handle_hook, header_value_from_map};
use opentelemetry::{
    sdk::{trace as sdktrace, Resource},
    KeyValue,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, instrument};

#[derive(Clone, Debug)]
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
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, Mikey and backendsouls!"
}

#[instrument]
async fn hook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> &'static str {
    debug!("Received request");
    return handle_hook(
        header_value_from_map(&headers),
        std::env::var("CIRCLECI_HOOK_SECRET_TOKEN").ok(),
        body.as_ref(),
        &state.tracer,
    )
    .await;
}
