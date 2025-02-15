use actix_web::HttpResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        match self {
            Error::StoreError(err) => match err {
                rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                rustical_store::Error::InvalidData(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Error::ChronoParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DavError(err) => err.status_code(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
            Error::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        error!("Error: {self}");
        match self {
            Error::DavError(err) => err.error_response(),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            match &self {
                Error::StoreError(err) => match err {
                    rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                    rustical_store::Error::InvalidData(_) => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                Error::ChronoParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                // Error::DavError(err) => err.status_code(),
                // TODO
                Error::DavError(err) => StatusCode::INTERNAL_SERVER_ERROR,
                Error::Unauthorized => StatusCode::UNAUTHORIZED,
                Error::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
                Error::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
                Error::NotFound => StatusCode::NOT_FOUND,
            },
            self.to_string(),
        )
            .into_response()
    }
}
