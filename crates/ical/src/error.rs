use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid ics/vcf input: {0}")]
    InvalidData(String),

    #[error("Missing calendar")]
    MissingCalendar,

    #[error("Missing contact")]
    MissingContact,

    #[error(transparent)]
    ParserError(#[from] ical::parser::ParserError),
}

impl Error {
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidData(_) | Self::MissingCalendar | Self::MissingContact => {
                StatusCode::BAD_REQUEST
            }
            Self::ParserError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
