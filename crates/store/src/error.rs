use axum::response::IntoResponse;
use http::StatusCode;
use tracing::error;

use crate::auth::InvalidPrincipalTypeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Resource already exists and overwrite=false")]
    AlreadyExists,

    #[error("Read-only")]
    ReadOnly,

    #[error("Invalid principal id: Id cannot contain ':' or '$'.")]
    InvalidPrincipalId,

    #[error(transparent)]
    InvalidPrincipalType(#[from] InvalidPrincipalTypeError),

    #[error("Error generating password hash")]
    PasswordHash,

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    IcalError(#[from] caldata::parser::ParserError),
}

impl Error {
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::CONFLICT,
            Self::ReadOnly => StatusCode::FORBIDDEN,
            Self::InvalidPrincipalId | Self::InvalidPrincipalType(_) => StatusCode::BAD_REQUEST,
            Self::IcalError(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        if matches!(
            self.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::CONFLICT
        ) {
            error!("{self}");
        }
        (self.status_code(), self.to_string()).into_response()
    }
}
