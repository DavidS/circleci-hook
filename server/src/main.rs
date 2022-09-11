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
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use std::{env, str::FromStr};
use tonic::{
    metadata::{MetadataKey, MetadataMap},
    transport::ClientTlsConfig,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, instrument};
use url::Url;

#[derive(Clone, Debug)]
struct AppState {
    tracer: sdktrace::Tracer,
}

const ENDPOINT: &str = "CIRCLECI_OTLP_ENDPOINT";
const HEADER_PREFIX: &str = "CIRCLECI_OTLP_";
const SECRET_TOKEN: &str = "CIRCLECI_HOOK_SECRET";
const SERVICE_NAME: &str = "CIRCLECI_HOOK_SERVICE";

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    let endpoint = env::var(ENDPOINT).unwrap_or_else(|_| {
        panic!(
            "You must specify and endpoint to connect to with the variable {:?}.",
            ENDPOINT
        )
    });
    let endpoint = Url::parse(&endpoint).expect("endpoint is not a valid url");
    env::remove_var(ENDPOINT);
    let mut metadata = MetadataMap::new();
    for (key, value) in env::vars()
        .filter(|(name, _)| name.starts_with(HEADER_PREFIX))
        .map(|(name, value)| {
            let header_name = name
                .strip_prefix(HEADER_PREFIX)
                .map(|h| h.replace('_', "-"))
                .map(|h| h.to_ascii_lowercase())
                .unwrap();
            (header_name, value)
        })
    {
        metadata.insert(MetadataKey::from_str(&key).unwrap(), value.parse().unwrap());
    }

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(metadata)
                .with_tls_config(
                    ClientTlsConfig::new().domain_name(
                        endpoint
                            .host_str()
                            .expect("the specified endpoint should have a valid host"),
                    ),
                ),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                env::var(SERVICE_NAME).unwrap_or_else(|_| "circleci".to_string()),
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let tracer = init_tracer().expect("build an OTLP tracer");
    let state = AppState { tracer };

    let app = Router::with_state(state)
        .route("/", get(root))
        .route("/", post(hook_handler))
        .layer(
            ServiceBuilder::new()
                // .layer(middleware::from_fn(validate_circleci_signature))
                .layer(TraceLayer::new_for_http()),
        );

    let addr = "[::]:3000".parse().unwrap();
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
    handle_hook(
        header_value_from_map(&headers),
        env::var(SECRET_TOKEN).ok(),
        body.as_ref(),
        &state.tracer,
    )
    .await
}
