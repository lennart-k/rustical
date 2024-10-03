use actix_web::{error::ErrorUnauthorized, FromRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub displayname: Option<String>,
    pub password: Option<String>,
}

impl FromRequest for User {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.extensions()
                .get::<User>()
                .cloned()
                .ok_or(ErrorUnauthorized("")),
        )
    }
}
