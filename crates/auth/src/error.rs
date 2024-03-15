use actix_web::{http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error, Clone)]
pub enum Error {
    #[display(fmt = "Internal server error")]
    InternalError,
    #[display(fmt = "Not found")]
    NotFound,
    #[display(fmt = "Bad request")]
    BadRequest,
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
                .body(self.to_string()),
            _ => HttpResponse::build(self.status_code()).body(self.to_string()),
        }
    }
}
