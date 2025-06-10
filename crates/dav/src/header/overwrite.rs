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

#[derive(Debug, PartialEq, Default)]
pub enum Overwrite {
    #[default]
    T,
    F,
}

impl Overwrite {
    pub fn is_true(&self) -> bool {
        matches!(self, Self::T)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Overwrite {
    type Rejection = InvalidOverwriteHeader;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(overwrite_header) = parts.headers.get("Overwrite") {
            overwrite_header.as_bytes().try_into()
        } else {
            Ok(Self::default())
        }
    }
}

impl TryFrom<&[u8]> for Overwrite {
    type Error = InvalidOverwriteHeader;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"T" => Ok(Overwrite::T),
            b"F" => Ok(Overwrite::F),
            _ => Err(InvalidOverwriteHeader),
        }
    }
}
