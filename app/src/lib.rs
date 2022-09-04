use http::HeaderMap;
use opentelemetry::sdk::trace::Tracer;
use signatures::{parse_signature_header, verify_signature};

use crate::payload::WebhookPayload;

pub mod signatures;
pub mod payload;

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
) -> &'static str {
    if let Some(key) = key {
        if let Some(signature_hex) =
            header_value.and_then(|header_value| parse_signature_header(header_value))
        {
            if !verify_signature(body, key.as_bytes(), signature_hex) {
                todo!("Deal with failing signature verification");
            }
        } else {
            todo!("Deal with missing header or failing signature parsing");
        }
    }

    if serde_json::from_slice::<WebhookPayload>(body.as_ref())
        .and_then(|payload| Ok(payload.build_span(tracer)))
        .is_ok()
    {
        return "Success!";
    } else {
        todo!("Error handling")
    }
}
