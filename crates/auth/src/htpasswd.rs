use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AuthInfo, CheckAuthentication};

#[derive(Debug)]
pub struct HtpasswdAuth {
    pub config: HtpasswdAuthConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HtpasswdAuthUserConfig {
    password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HtpasswdAuthConfig {
    pub users: HashMap<String, HtpasswdAuthUserConfig>,
}

impl CheckAuthentication for HtpasswdAuth {
    type Error = crate::error::Error;
    type Future = Ready<Result<AuthInfo, Self::Error>>;

    fn validate(&self, req: &HttpRequest) -> Self::Future {
        if let Ok(auth) = Authorization::<Basic>::parse(req) {
            let user_id = auth.as_ref().user_id();
            // Map None to empty password
            let password = auth.as_ref().password().unwrap_or_default();

            let user_config = if let Some(user_config) = self.config.users.get(user_id) {
                user_config
            } else {
                return err(crate::error::Error::Unauthorized);
            };

            if let Err(e) = password_auth::verify_password(password, &user_config.password) {
                dbg!(e);
                return err(crate::error::Error::Unauthorized);
            }

            ok(AuthInfo {
                user_id: user_id.to_string(),
            })
        } else {
            err(crate::error::Error::Unauthorized)
        }
    }
}
