use actix_web::{HttpResponse, http::StatusCode};
use rustical_xml::XmlError;
use thiserror::Error;
use tracing::error;

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
    XmlError(#[from] rustical_xml::XmlError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Precondition Failed")]
    PreconditionFailed,
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::XmlError(error) => match &error {
                XmlError::InvalidTag(..)
                | XmlError::MissingField(_)
                | XmlError::UnsupportedEvent(_)
                | XmlError::InvalidVariant(_)
                | XmlError::InvalidFieldName(_, _)
                | XmlError::InvalidValue(_) => StatusCode::UNPROCESSABLE_ENTITY,
                _ => StatusCode::BAD_REQUEST,
            },
            Error::PropReadOnly => StatusCode::CONFLICT,
            Error::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            Self::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        error!("Error: {self}");
        match self {
            Error::Unauthorized => HttpResponse::build(self.status_code())
                .append_header(("WWW-Authenticate", "Basic"))
                .body(self.to_string()),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
