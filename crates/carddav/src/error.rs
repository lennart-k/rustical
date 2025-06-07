use actix_web::{HttpResponse, http::StatusCode};
use tracing::error;

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

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::StoreError(err) => match err {
                rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                rustical_store::Error::AlreadyExists => StatusCode::CONFLICT,
                rustical_store::Error::ReadOnly => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Error::ChronoParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DavError(err) => StatusCode::try_from(err.status_code().as_u16())
                .expect("Just converting between versions"),
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
            Error::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Self::IcalError(err) => err.status_code(),
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        error!("Error: {self}");
        match self {
            Error::DavError(err) => err.error_response(),
            Error::IcalError(err) => err.error_response(),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
