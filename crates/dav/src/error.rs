use actix_web::{http::StatusCode, HttpResponse};
use thiserror::Error;

// use crate::routes::propfind;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error :(")]
    InternalError,

    #[error("prop is read-only")]
    PropReadOnly,

    #[error(transparent)]
    XmlDeserializationError(#[from] quick_xml::DeError),

    #[error(transparent)]
    XmlSerializationError(#[from] quick_xml::SeError),

    #[error("Internal server error")]
    Other(#[from] anyhow::Error),
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::XmlDeserializationError(_) => StatusCode::BAD_REQUEST,
            Self::XmlSerializationError(_) => StatusCode::BAD_REQUEST,
            Error::PropReadOnly => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Unauthorized => HttpResponse::build(self.status_code())
                .append_header(("WWW-Authenticate", "Basic"))
                .body(self.to_string()),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
