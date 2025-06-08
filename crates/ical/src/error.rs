use axum::{http::StatusCode, response::IntoResponse};

use crate::CalDateTimeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid ics/vcf input: {0}")]
    InvalidData(String),

    #[error("Missing calendar")]
    MissingCalendar,

    #[error("Missing contact")]
    MissingContact,

    #[error(transparent)]
    ParserError(#[from] ical::parser::ParserError),

    #[error(transparent)]
    CalDateTimeError(#[from] CalDateTimeError),

    #[error(transparent)]
    RRuleError(#[from] rrule::RRuleError),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidData(_) => StatusCode::BAD_REQUEST,
            Self::MissingCalendar | Self::MissingContact => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
