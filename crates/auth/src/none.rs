use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};

use crate::error::Error;

use super::{AuthInfo, CheckAuthentication};

#[derive(Debug, Clone)]
pub struct NoneAuth;

impl CheckAuthentication for NoneAuth {
    fn validate(&self, req: &HttpRequest) -> Result<AuthInfo, Error> {
        if let Ok(auth) = Authorization::<Basic>::parse(req) {
            Ok(AuthInfo {
                user_id: auth.as_ref().user_id().to_string(),
            })
        } else {
            Err(crate::error::Error::Unauthorized)
        }
    }
}
