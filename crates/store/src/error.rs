use axum::response::IntoResponse;
use http::StatusCode;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Resource already exists and overwrite=false")]
    AlreadyExists,

    #[error("Invalid principal type: {0}")]
    InvalidPrincipalType(String),

    #[error("Read-only")]
    ReadOnly,

    #[error("Error generating password hash")]
    PasswordHash,

    #[error(transparent)]
    IO(#[from] std::io::Error),

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
            Self::IcalError(_err) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidPrincipalType(_) => StatusCode::BAD_REQUEST,
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
