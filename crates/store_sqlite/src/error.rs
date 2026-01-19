use tracing::warn;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SqlxError(sqlx::Error),

    #[error(transparent)]
    StoreError(rustical_store::Error),

    #[error(transparent)]
    IcalError(#[from] rustical_ical::Error),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::StoreError(rustical_store::Error::NotFound),
            sqlx::Error::Database(err) => {
                if err.is_unique_violation() {
                    warn!("{err}");
                    Self::StoreError(rustical_store::Error::AlreadyExists)
                } else {
                    Self::SqlxError(sqlx::Error::Database(err))
                }
            }
            err => Self::SqlxError(err),
        }
    }
}

impl From<Error> for rustical_store::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::SqlxError(err) => Self::Other(err.into()),
            Error::IcalError(err) => Self::Other(err.into()),
            Error::StoreError(err) => err,
        }
    }
}

impl From<rustical_store::Error> for Error {
    fn from(value: rustical_store::Error) -> Self {
        Self::StoreError(value)
    }
}
