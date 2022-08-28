use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn validate_circleci_signature<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("circleci-signature")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if get_signature_hash(auth_header).is_some() => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn get_signature_hash<'a>(header_value: &str) -> Option<String> {
    for signature in header_value.split(",") {
        let splits: Vec<&str> = signature.split("=").collect();
        if splits.len() != 2 {
            println!(
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
mod tests {
    use super::{get_signature_hash};

    #[test]
    fn test_empty() {
        assert_eq!(get_signature_hash(""), None);
    }

    #[test]
    fn test_only_v1_signature() {
        assert_eq!(get_signature_hash("v1=foobar"), Some("foobar".to_string()));
    }

    #[test]
    fn test_multiple_signatures() {
        assert_eq!(
            get_signature_hash("v1=foobar,v2=wibble"),
            Some("foobar".to_string())
        );
    }
}
