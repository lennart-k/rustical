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

    #[error("Forbidden")]
    Forbidden,

    #[error(transparent)]
    ShareRejected(#[from] crate::share::ShareRejection),

    #[error("The object changed since it was read")]
    PreconditionFailed,

    #[error(transparent)]
    CopyRule(#[from] crate::share::CopyRule),

    #[error("Invalid visibility levels")]
    InvalidVisibility,

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
            Self::ReadOnly | Self::Forbidden | Self::CopyRule(crate::share::CopyRule::Level) => {
                StatusCode::FORBIDDEN
            }
            // Other writes through a share that break a rule of the owner's calendar
            Self::AlreadyExists | Self::CopyRule(_) => StatusCode::CONFLICT,
            Self::InvalidPrincipalId
            | Self::InvalidPrincipalType(_)
            | Self::ShareRejected(_)
            | Self::InvalidVisibility => StatusCode::BAD_REQUEST,
            Self::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
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
