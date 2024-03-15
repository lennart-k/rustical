use actix_web::HttpRequest;

use crate::error::Error;
pub use extractor::AuthInfoExtractor;
pub use htpasswd::{HtpasswdAuth, HtpasswdAuthConfig};
pub use none::NoneAuth;
pub mod error;
pub mod extractor;
pub mod htpasswd;
pub mod none;

#[derive(Clone)]
pub struct AuthInfo {
    pub user_id: String,
}

pub trait CheckAuthentication: Send + Sync + 'static {
    fn validate(&self, req: &HttpRequest) -> Result<AuthInfo, Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum AuthProvider {
    Htpasswd(HtpasswdAuth),
    None(NoneAuth),
}

impl CheckAuthentication for AuthProvider {
    fn validate(&self, req: &HttpRequest) -> Result<AuthInfo, Error>
    where
        Self: Sized,
    {
        match self {
            Self::Htpasswd(auth) => auth.validate(req),
            Self::None(auth) => auth.validate(req),
        }
    }
}
