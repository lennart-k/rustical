#[cfg(feature = "actix")]
use actix_web::{HttpRequest, ResponseError};
#[cfg(feature = "axum")]
use axum::{body::Body, extract::FromRequestParts, response::IntoResponse};
use futures_util::future::{Ready, err, ok};
use rustical_xml::{ValueDeserialize, ValueSerialize, XmlError};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid Depth header")]
pub struct InvalidDepthHeader;

#[cfg(feature = "actix")]
impl ResponseError for InvalidDepthHeader {
    fn status_code(&self) -> actix_web::http::StatusCode {
        http_02::StatusCode::BAD_REQUEST
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for InvalidDepthHeader {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .expect("this always works")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Depth {
    Zero,
    One,
    Infinity,
}

impl ValueSerialize for Depth {
    fn serialize(&self) -> String {
        match self {
            Depth::Zero => "0",
            Depth::One => "1",
            Depth::Infinity => "infinity",
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
            b"0" => Ok(Depth::Zero),
            b"1" => Ok(Depth::One),
            b"Infinity" | b"infinity" => Ok(Depth::Infinity),
            _ => Err(InvalidDepthHeader),
        }
    }
}

#[cfg(feature = "actix")]
impl actix_web::FromRequest for Depth {
    type Error = InvalidDepthHeader;
    type Future = Ready<Result<Self, Self::Error>>;

    fn extract(req: &HttpRequest) -> Self::Future {
        if let Some(depth_header) = req.headers().get("Depth") {
            match depth_header.as_bytes().try_into() {
                Ok(depth) => ok(depth),
                Err(e) => err(e),
            }
        } else {
            // default depth
            ok(Depth::Zero)
        }
    }

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        Self::extract(req)
    }
}

#[cfg(feature = "axum")]
impl<S: Send + Sync> FromRequestParts<S> for Depth {
    type Rejection = InvalidDepthHeader;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(depth_header) = parts.headers.get("Depth") {
            depth_header.as_bytes().try_into()
        } else {
            Ok(Self::Zero)
        }
    }
}
