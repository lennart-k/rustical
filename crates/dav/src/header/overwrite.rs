use axum::{body::Body, extract::FromRequestParts, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid Overwrite header")]
pub struct InvalidOverwriteHeader;

impl IntoResponse for InvalidOverwriteHeader {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::BAD_REQUEST)
            .body(Body::new("Invalid Overwrite header".to_string()))
            .expect("this always works")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Overwrite(pub bool);

impl Default for Overwrite {
    fn default() -> Self {
        Self(true)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Overwrite {
    type Rejection = InvalidOverwriteHeader;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.headers.get("Overwrite").map_or_else(
            || Ok(Self::default()),
            |overwrite_header| overwrite_header.as_bytes().try_into(),
        )
    }
}

impl TryFrom<&[u8]> for Overwrite {
    type Error = InvalidOverwriteHeader;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"T" => Ok(Self(true)),
            b"F" => Ok(Self(false)),
            _ => Err(InvalidOverwriteHeader),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{extract::FromRequestParts, response::IntoResponse};
    use http::Request;

    use crate::header::Overwrite;

    #[tokio::test]
    async fn test_overwrite_default() {
        let request = Request::put("asd").body(()).unwrap();
        let (mut parts, ()) = request.into_parts();
        let overwrite = Overwrite::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(
            Overwrite(true),
            overwrite,
            "By default we want to overwrite!"
        );
    }

    #[test]
    fn test_overwrite() {
        assert_eq!(
            Overwrite(true),
            Overwrite::try_from(b"T".as_slice()).unwrap()
        );
        assert_eq!(
            Overwrite(false),
            Overwrite::try_from(b"F".as_slice()).unwrap()
        );
        if let Err(err) = Overwrite::try_from(b"aslkdjlad".as_slice()) {
            let _ = err.into_response();
        } else {
            unreachable!("should return error")
        }
    }
}
