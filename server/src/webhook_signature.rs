use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use thiserror::Error;

pub async fn validate_circleci_signature<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("circleci-signature")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if parse_signature(auth_header).is_ok() => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Error, Debug, PartialEq)]
enum SignatureParseError {
    #[error("missing signature")]
    MissingSignature(),
    #[error("syntax error")]
    Syntax(),
    #[error("not implemented")]
    NotImplemented(),
}

fn parse_signature<'a>(header_value: &str) -> Result<String, SignatureParseError> {
    let splits: Vec<&str> = header_value.split("=").collect();
    if splits.len() != 2 {
        println!(
            "Invalid header `{}`, does contain {} parts!",
            header_value,
            splits.len()
        );
        return Err(SignatureParseError::Syntax());
    }

    if splits[0] == "v1" {
        return Ok(splits[1].to_string());
    }

    Err(SignatureParseError::NotImplemented())
}

#[cfg(test)]
mod tests {
    use super::{parse_signature, SignatureParseError};

    #[test]
    fn test_empty() {
        assert_eq!(parse_signature(""), Err(SignatureParseError::Syntax()));
    }

    #[test]
    fn test_only_v1_signature() {
        assert_eq!(parse_signature("v1=foobar"), Ok("foobar".to_string()));
    }
}
