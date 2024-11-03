use actix_web::{
    body::BoxBody,
    http::{header, StatusCode},
    FromRequest, HttpMessage, HttpResponse, ResponseError,
};
use derive_more::{derive::Deref, Display};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub displayname: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, Display)]
pub struct UnauthorizedError;

impl ResponseError for UnauthorizedError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(StatusCode::UNAUTHORIZED)
            .insert_header((
                header::WWW_AUTHENTICATE,
                r#"Basic realm="RustiCal", charset="UTF-8""#,
            ))
            // The force_close is a workaround for a bug where something freezes when the
            // connection is reused after a 401.
            // possibly related to https://github.com/actix/actix-web/issues/1805
            .force_close()
            .finish()
    }
}

impl FromRequest for User {
    type Error = UnauthorizedError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.extensions()
                .get::<Self>()
                .cloned()
                .ok_or(UnauthorizedError),
        )
    }
}
