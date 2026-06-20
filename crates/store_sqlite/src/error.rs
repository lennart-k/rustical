use axum::{http::StatusCode, response::IntoResponse};
use tracing::warn;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SqlxError(sqlx::Error),

    #[error(transparent)]
    StoreError(rustical_store::Error),

    #[error(transparent)]
    DavPushError(#[from] rustical_dav_push::Error),

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
            Error::DavPushError(err) => Self::Other(err.into()),
            Error::StoreError(err) => err,
        }
    }
}

impl From<rustical_store::Error> for Error {
    fn from(value: rustical_store::Error) -> Self {
        Self::StoreError(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        if let Self::StoreError(err) = self {
            return err.into_response();
        }

        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
