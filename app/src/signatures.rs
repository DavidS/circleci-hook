use hmac::{digest::FixedOutput, Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{debug, warn};

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

pub fn verify_signature(body: &[u8], key: &[u8], signature_hex: String) -> bool {
    let signature = hex::decode(signature_hex).expect("Decoding failed");
    debug!(
        "VERIFYING: body={:?}, key={:?}, signature={:?}",
        body, key, signature
    );
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(body);
    let result = mac.finalize_fixed();
    if result.ct_eq(&signature).into() {
        // debug!("SUCCESS!");
        return true;
    }
    warn!("FAILED signature verification: {:?}", result);
    false
}

#[cfg(test)]
mod verification_tests {
    // examples from the circleci docs:
    // Body	Secret Key	Signature
    // hello world	secret	734cc62f32841568f45715aeb9f4d7891324e6d948e4c6c60c0621cdac48623a
    // lalala	another-secret	daa220016c8f29a8b214fbfc3671aeec2145cfb1e6790184ffb38b6d0425fa00
    // an-important-request-payload	hunter123	9be2242094a9a8c00c64306f382a7f9d691de910b4a266f67bd314ef18ac49fa

    use super::verify_signature;

    #[test]
    fn test_hashes() {
        assert!(verify_signature(
            b"hello world",
            b"secret",
            "734cc62f32841568f45715aeb9f4d7891324e6d948e4c6c60c0621cdac48623a".to_string()
        ));
        assert!(verify_signature(
            b"lalala",
            b"another-secret",
            "daa220016c8f29a8b214fbfc3671aeec2145cfb1e6790184ffb38b6d0425fa00".to_string()
        ));
        assert!(verify_signature(
            b"an-important-request-payload",
            b"hunter123",
            "9be2242094a9a8c00c64306f382a7f9d691de910b4a266f67bd314ef18ac49fa".to_string()
        ));
    }
}

pub fn parse_signature_header(header_value: &str) -> Option<String> {
    for signature in header_value.split(',') {
        let splits: Vec<&str> = signature.split('=').collect();
        if splits.len() != 2 {
            warn!(
                "Invalid header `{}`, does contain {} parts!",
                header_value,
                splits.len()
            );
            return None;
        }

        if splits[0] == "v1" {
            return Some(splits[1].to_string());
        }
    }
    None
}

#[cfg(test)]
mod parse_tests {
    use super::parse_signature_header;

    #[test]
    fn test_empty() {
        assert_eq!(parse_signature_header(""), None);
    }

    #[test]
    fn test_only_v1_signature() {
        assert_eq!(
            parse_signature_header("v1=foobar"),
            Some("foobar".to_string())
        );
    }

    #[test]
    fn test_multiple_signatures() {
        assert_eq!(
            parse_signature_header("v1=foobar,v2=wibble"),
            Some("foobar".to_string())
        );
    }
}
