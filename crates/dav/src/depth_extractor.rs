use actix_web::{http::StatusCode, FromRequest, HttpRequest, ResponseError};
use derive_more::Display;
use futures_util::future::{err, ok, Ready};

#[derive(Debug, Display)]
pub struct InvalidDepthHeader {}

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
            _ => Err(InvalidDepthHeader {}),
        }
    }
}

impl FromRequest for Depth {
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
