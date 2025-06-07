use http::StatusCode;
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

impl Error {
    pub fn status_code(&self) -> StatusCode {
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
}

#[cfg(feature = "actix")]
impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status_code()
            .as_u16()
            .try_into()
            .expect("Just converting between versions")
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::ResponseError;

        error!("Error: {self}");
        match self {
            Error::Unauthorized => actix_web::HttpResponse::build(ResponseError::status_code(self))
                .append_header(("WWW-Authenticate", "Basic"))
                .body(self.to_string()),
            _ => actix_web::HttpResponse::build(ResponseError::status_code(self))
                .body(self.to_string()),
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::body::Body;

        let mut resp = axum::response::Response::builder().status(self.status_code());
        if matches!(&self, &Error::Unauthorized) {
            resp.headers_mut()
                .expect("This must always work")
                .insert("WWW-Authenticate", "Basic".parse().unwrap());
        }

        resp.body(Body::new(self.to_string()))
            .expect("This should always work")
    }
}
