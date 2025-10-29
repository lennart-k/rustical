use axum::body::Body;
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

    #[error("Forbidden")]
    Forbidden,
}

impl Error {
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
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
            Self::PropReadOnly => StatusCode::CONFLICT,
            Self::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            Self::InternalError | Self::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        if matches!(
            self.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::PRECONDITION_FAILED
        ) {
            error!("{self}");
        }

        let mut resp = axum::response::Response::builder().status(self.status_code());
        if matches!(&self, &Self::Unauthorized) {
            resp.headers_mut()
                .expect("This must always work")
                .insert("WWW-Authenticate", "Basic".parse().unwrap());
        }

        resp.body(Body::new(self.to_string()))
            .expect("This should always work")
    }
}
