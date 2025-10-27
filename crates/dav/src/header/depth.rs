use axum::{
    body::Body,
    extract::{FromRequestParts, OptionalFromRequestParts},
    response::IntoResponse,
};
use rustical_xml::{ValueDeserialize, ValueSerialize, XmlError};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid Depth header")]
pub struct InvalidDepthHeader;

impl IntoResponse for InvalidDepthHeader {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .expect("this always works")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Depth {
    Zero,
    One,
    Infinity,
}

impl ValueSerialize for Depth {
    fn serialize(&self) -> String {
        match self {
            Self::Zero => "0",
            Self::One => "1",
            Self::Infinity => "infinity",
        }
        .to_owned()
    }
}

impl ValueDeserialize for Depth {
    fn deserialize(val: &str) -> Result<Self, XmlError> {
        match val {
            "0" => Ok(Self::Zero),
            "1" => Ok(Self::One),
            "infinity" | "Infinity" => Ok(Self::Infinity),
            _ => Err(XmlError::InvalidVariant(
                "Invalid value for depth".to_owned(),
            )),
        }
    }
}

impl TryFrom<&[u8]> for Depth {
    type Error = InvalidDepthHeader;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"0" => Ok(Self::Zero),
            b"1" => Ok(Self::One),
            b"Infinity" | b"infinity" => Ok(Self::Infinity),
            _ => Err(InvalidDepthHeader),
        }
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for Depth {
    type Rejection = InvalidDepthHeader;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        if let Some(depth_header) = parts.headers.get("Depth") {
            Ok(Some(depth_header.as_bytes().try_into()?))
        } else {
            Ok(None)
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
            Ok(Self::Zero)
        }
    }
}
