use http::StatusCode;

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
    IcalError(#[from] rustical_ical::Error),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::CONFLICT,
            Self::ReadOnly => StatusCode::FORBIDDEN,
            Self::IcalError(err) => err.status_code(),
            Self::InvalidPrincipalType(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
