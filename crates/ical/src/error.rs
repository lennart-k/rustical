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

#[cfg(feature = "actix")]
impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::InvalidData(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Self::MissingCalendar | Self::MissingContact => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
