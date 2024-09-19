use actix_web::{http::StatusCode, HttpResponse};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("Internal server error")]
    InternalError,
    #[error("Not found")]
    NotFound,
    #[error("Bad request")]
    BadRequest,
    #[error("Unauthorized")]
    Unauthorized,
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Unauthorized => HttpResponse::build(self.status_code())
                .append_header(("WWW-Authenticate", "Basic"))
                // This is an unfortunate workaround for https://github.com/actix/actix-web/issues/1805
                .force_close()
                .body(self.to_string()),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
