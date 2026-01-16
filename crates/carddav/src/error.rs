use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not Found")]
    NotFound,

    #[error("Not implemented")]
    NotImplemented,

    #[error(transparent)]
    StoreError(#[from] rustical_store::Error),

    #[error(transparent)]
    ChronoParseError(#[from] chrono::ParseError),

    #[error(transparent)]
    DavError(#[from] rustical_dav::Error),

    #[error(transparent)]
    XmlDecodeError(#[from] rustical_xml::XmlError),

    #[error(transparent)]
    IcalError(#[from] rustical_ical::Error),
}

impl Error {
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::StoreError(err) => match err {
                rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                rustical_store::Error::AlreadyExists => StatusCode::CONFLICT,
                rustical_store::Error::ReadOnly => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::DavError(err) => err.status_code(),
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
            Self::ChronoParseError(_) | Self::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            // TODO: Can also be Bad Request, if it's used input
            Self::IcalError(_err) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
