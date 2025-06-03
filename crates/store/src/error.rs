use actix_web::{ResponseError, http::StatusCode};
use rustical_ical::CalDateTimeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Resource already exists and overwrite=false")]
    AlreadyExists,

    #[error("Invalid ics/vcf input: {0}")]
    InvalidData(String),

    #[error("Read-only")]
    ReadOnly,

    #[error("Error generating password hash")]
    PasswordHash,

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    ParserError(#[from] ical::parser::ParserError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    CalDateTimeError(#[from] CalDateTimeError),

    #[error(transparent)]
    RRuleError(#[from] rrule::RRuleError),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::CONFLICT,
            Self::InvalidData(_) => StatusCode::BAD_REQUEST,
            Self::ReadOnly => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
