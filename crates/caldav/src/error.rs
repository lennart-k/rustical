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
                // TODO: change
                Error::DavError(_err) => StatusCode::INTERNAL_SERVER_ERROR,
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
