#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Resource already exists and overwrite=false")]
    AlreadyExists,

    #[error("Invalid ics/vcf input: {0}")]
    InvalidData(String),

    #[error(transparent)]
    SqlxError(sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Error::NotFound,
            err => Error::SqlxError(err),
        }
    }
}

// TODO: clean up error types
impl From<Error> for rustical_store::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::NotFound => Self::NotFound,
            Error::AlreadyExists => Self::AlreadyExists,
            Error::InvalidData(b) => Self::InvalidData(b),
            Error::SqlxError(err) => Self::Other(err.into()),
            Error::Other(err) => Self::Other(err),
        }
    }
}

impl From<rustical_store::Error> for Error {
    fn from(value: rustical_store::Error) -> Self {
        match value {
            rustical_store::Error::NotFound => Self::NotFound,
            rustical_store::Error::AlreadyExists => Self::AlreadyExists,
            rustical_store::Error::InvalidData(b) => Self::InvalidData(b),
            rustical_store::Error::Other(err) => Self::Other(err),
            rustical_store::Error::ParserError(err) => Self::Other(err.into()),
        }
    }
}
