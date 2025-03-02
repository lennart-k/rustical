use axum::{extract::FromRequestParts, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("Invalid Depth header")]
pub struct InvalidDepthHeader;

impl IntoResponse for InvalidDepthHeader {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

#[derive(Debug, PartialEq)]
pub enum Depth {
    Zero,
    One,
    Infinity,
}

impl TryFrom<&[u8]> for Depth {
    type Error = InvalidDepthHeader;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"0" => Ok(Depth::Zero),
            b"1" => Ok(Depth::One),
            b"Infinity" | b"infinity" => Ok(Depth::Infinity),
            _ => Err(InvalidDepthHeader),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Depth {
    type Rejection = InvalidDepthHeader;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(depth_header) = parts.headers.get("Depth") {
            depth_header.as_bytes().try_into()
        } else {
            // default depth
            Ok(Depth::Zero)
        }
    }
}
