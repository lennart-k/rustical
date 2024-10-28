#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SqlxError(sqlx::Error),

    #[error(transparent)]
    StoreError(rustical_store::Error),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Error::StoreError(rustical_store::Error::NotFound),
            err => Error::SqlxError(err),
        }
    }
}

impl From<Error> for rustical_store::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::SqlxError(err) => Self::Other(err.into()),
            Error::StoreError(err) => err,
        }
    }
}

impl From<rustical_store::Error> for Error {
    fn from(value: rustical_store::Error) -> Self {
        Self::StoreError(value)
    }
}
