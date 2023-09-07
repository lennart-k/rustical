use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use futures_util::future::{err, ok, Ready};

use super::{AuthInfo, CheckAuthentication};

#[derive(Debug, Clone)]
pub struct NoneAuth;

impl CheckAuthentication for NoneAuth {
    type Error = crate::error::Error;
    type Future = Ready<Result<AuthInfo, Self::Error>>;

    fn validate(&self, req: &HttpRequest) -> Self::Future {
        if let Ok(auth) = Authorization::<Basic>::parse(req) {
            ok(AuthInfo {
                user_id: auth.as_ref().user_id().to_string(),
            })
        } else {
            err(crate::error::Error::Unauthorized)
        }
    }
}
