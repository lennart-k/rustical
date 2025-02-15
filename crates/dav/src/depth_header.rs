use actix_web::{http::StatusCode, HttpRequest, ResponseError};
use axum::{
    extract::{FromRequest, FromRequestParts},
    response::{IntoResponse, IntoResponseParts, ResponseParts},
};
use futures_util::future::{err, ok, Ready};
use std::convert::Infallible;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("Invalid Depth header")]
pub struct InvalidDepthHeader;

impl IntoResponse for InvalidDepthHeader {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

impl ResponseError for InvalidDepthHeader {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::BAD_REQUEST
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
