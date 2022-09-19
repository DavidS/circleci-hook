use arrayref::array_ref;
use http::HeaderMap;
use opentelemetry::{
    sdk::trace::Tracer,
    trace::{SpanId, TraceFlags, TraceId},
};
use signatures::{parse_signature_header, verify_signature};
use thiserror::Error;
use uuid::Uuid;

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

pub fn translate_traceparent(workflow_id: Uuid, job_id: Uuid) -> String {
    // From https://github.com/open-telemetry/opentelemetry-rust/blob/d4b9befea04bcc7fc19319a6ebf5b5070131c486/opentelemetry-sdk/src/propagation/trace_context.rs#L117-L121
    let supported_version: u8 = 0;
    let trace_id = TraceId::from_bytes(*workflow_id.as_bytes());
    let span_id = SpanId::from_bytes(*array_ref!(job_id.as_bytes(), 0, 8));

    let header_value = format!(
        "{:02x}-{:032x}-{:016x}-{:02x}",
        supported_version,
        trace_id,
        span_id,
        TraceFlags::SAMPLED
    );
    format!("export TRACEPARENT={}", header_value)
}

#[cfg(test)]
mod tests_traceparent {
    use uuid::Uuid;

    use super::translate_traceparent;

    #[test]
    fn traceparent() {
        let workflow_id = Uuid::parse_str("cedf175c-c537-49f8-9b5c-95892f0b2407").unwrap();
        let job_id = Uuid::parse_str("3f6a7c4a-0bb6-463a-bd0c-3d5e0b1b1d34").unwrap();
        assert_eq!(
            translate_traceparent(workflow_id, job_id),
            "export TRACEPARENT=00-cedf175cc53749f89b5c95892f0b2407-3f6a7c4a0bb6463a-01"
        );
    }
}
