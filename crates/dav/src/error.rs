use axum::body::Body;
use axum::http::header::WWW_AUTHENTICATE;
use axum::http::StatusCode;
use axum::response::AppendHeaders;
use axum::{http::Response, response::IntoResponse};
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
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        if matches!(&self, Error::Unauthorized) {
            return (
                StatusCode::UNAUTHORIZED,
                AppendHeaders([(WWW_AUTHENTICATE, "Basic")]),
                self.to_string(),
            )
                .into_response();
        }
        let status_code = match &self {
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
            Self::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}
