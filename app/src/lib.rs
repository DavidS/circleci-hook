use http::HeaderMap;
use opentelemetry::sdk::trace::Tracer;
use signatures::{parse_signature_header, verify_signature};
use thiserror::Error;

use crate::payload::WebhookPayload;

pub mod payload;
pub mod signatures;

#[derive(Error, Debug)]
pub enum HookError {
    #[error("signature verification failed")]
    SignatureVerification,
    #[error("signature header not found")]
    HeaderMissing,
    #[error("deserialization failed")]
    DeserializationFailed(#[from] serde_json::Error),
    #[error("unknown hook error")]
    Unknown,
}

pub fn header_value_from_map(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("circleci-signature")
        .and_then(|header| header.to_str().ok())
}

pub async fn handle_hook(
    header_value: Option<&str>,
    key: Option<String>,
    body: &[u8],
    tracer: &Tracer,
) -> Result<&'static str, HookError> {
    if let Some(key) = key {
        if let Some(signature_hex) = header_value.and_then(parse_signature_header) {
            if !verify_signature(body, key.as_bytes(), signature_hex) {
                return Err(HookError::SignatureVerification);
            }
        } else {
            return Err(HookError::HeaderMissing);
        }
    }

    serde_json::from_slice::<WebhookPayload>(body)
        .map(|payload| {
            payload.build_span(tracer);
            "Success!"
        })
        .map_err(HookError::DeserializationFailed)
}
