use actix_web::{
    HttpResponse,
    http::{StatusCode, header::ContentType},
};
use rustical_xml::{XmlSerialize, XmlSerializeRoot};
use tracing::error;

#[derive(Debug, thiserror::Error, XmlSerialize)]
pub enum Precondition {
    #[error("valid-calendar-data")]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    ValidCalendarData,
}

impl actix_web::ResponseError for Precondition {
    fn status_code(&self) -> StatusCode {
        StatusCode::PRECONDITION_FAILED
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let mut output: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
        let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);

        let error = rustical_dav::xml::ErrorElement(self);
        if let Err(err) = error.serialize_root(&mut writer) {
            return rustical_dav::Error::from(err).error_response();
        }

        HttpResponse::PreconditionFailed()
            .content_type(ContentType::xml())
            .body(String::from_utf8(output).unwrap())
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
    IcalError(#[from] rustical_ical::Error),

    #[error(transparent)]
    PreconditionFailed(Precondition),
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::StoreError(err) => match err {
                rustical_store::Error::NotFound => StatusCode::NOT_FOUND,
                rustical_store::Error::AlreadyExists => StatusCode::CONFLICT,
                rustical_store::Error::ReadOnly => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Error::ChronoParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DavError(err) => StatusCode::try_from(err.status_code().as_u16())
                .expect("Just converting between versions"),
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::XmlDecodeError(_) => StatusCode::BAD_REQUEST,
            Error::NotImplemented => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::IcalError(err) => err.status_code(),
            Error::PreconditionFailed(err) => err.status_code(),
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        error!("Error: {self}");
        match self {
            Error::DavError(err) => err.error_response(),
            Error::IcalError(err) => err.error_response(),
            Error::PreconditionFailed(err) => err.error_response(),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
