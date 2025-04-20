use actix_web::{FromRequest, HttpRequest, ResponseError, http::StatusCode};
use futures_util::future::{Ready, err, ok};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid Overwrite header")]
pub struct InvalidOverwriteHeader;

impl ResponseError for InvalidOverwriteHeader {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::BAD_REQUEST
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

impl FromRequest for Overwrite {
    type Error = InvalidOverwriteHeader;
    type Future = Ready<Result<Self, Self::Error>>;

    fn extract(req: &HttpRequest) -> Self::Future {
        if let Some(overwrite_header) = req.headers().get("Overwrite") {
            match overwrite_header.as_bytes().try_into() {
                Ok(depth) => ok(depth),
                Err(e) => err(e),
            }
        } else {
            // default depth
            ok(Overwrite::F)
        }
    }

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        Self::extract(req)
    }
}
