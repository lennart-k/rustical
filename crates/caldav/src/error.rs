use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use headers::{ContentType, HeaderMapExt};
use http::StatusCode;
use rustical_xml::{XmlSerialize, XmlSerializeRoot};
use tracing::error;

#[derive(Debug, thiserror::Error, XmlSerialize)]
pub enum Precondition {
    #[error("valid-calendar-data")]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    ValidCalendarData,
    #[error("calendar-timezone")]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarTimezone(&'static str),
}

impl IntoResponse for Precondition {
    fn into_response(self) -> axum::response::Response {
        let mut output: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
        let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);

        let error = rustical_dav::xml::ErrorElement(&self);
        if let Err(err) = error.serialize_root(&mut writer) {
            return rustical_dav::Error::from(err).into_response();
        }
        let mut res = Response::builder().status(StatusCode::FORBIDDEN);
        res.headers_mut().unwrap().typed_insert(ContentType::xml());
        res.body(Body::from(output)).unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not Found")]
    NotFound,

    #[error("Not implemented")]
    NotImplemented,

    #[error(transparent)]
    StoreError(#[from] rustical_store::Error),

    #[error(transparent)]
    ChronoParseError(#[from] chrono::ParseError),

    #[error(transparent)]
    DavError(#[from] rustical_dav::Error),

    #[error(transparent)]
    XmlDecodeError(#[from] rustical_xml::XmlError),

    #[error(transparent)]
    PreconditionFailed(Precondition),
}

impl Error {
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::StoreError(err) => match err {
                rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                rustical_store::Error::AlreadyExists => StatusCode::CONFLICT,
                rustical_store::Error::ReadOnly => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::DavError(err) => StatusCode::try_from(err.status_code().as_u16())
                .expect("Just converting between versions"),
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
            Self::ChronoParseError(_) | Self::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            // The correct status code for a failed precondition is not PreconditionFailed but
            // Forbidden (or Conflict):
            // https://datatracker.ietf.org/doc/html/rfc4791#section-1.3
            Self::PreconditionFailed(_err) => StatusCode::FORBIDDEN,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        if let Self::PreconditionFailed(precondition) = self {
            return precondition.into_response();
        }
        if matches!(self.status_code(), StatusCode::INTERNAL_SERVER_ERROR) {
            error!("{self}");
        }
        (self.status_code(), self.to_string()).into_response()
    }
}
