#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Invalid ics input: {0}")]
    InvalidIcs(String),

    #[error(transparent)]
    SqlxError(sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    ParserError(#[from] ical::parser::ParserError),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Error::NotFound,
            err => Error::SqlxError(err),
        }
    }
}
