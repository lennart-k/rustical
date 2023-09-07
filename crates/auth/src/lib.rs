use actix_web::{HttpRequest, ResponseError};
use futures_util::{future::Ready, Future};

pub use extractor::AuthInfoExtractor;
pub use htpasswd::{HtpasswdAuth, HtpasswdAuthConfig};
pub use none::NoneAuth;
pub mod error;
pub mod extractor;
pub mod htpasswd;
pub mod none;

pub struct AuthInfo {
    pub user_id: String,
}

pub trait CheckAuthentication: Send + Sync + 'static {
    type Error: ResponseError;
    type Future: Future<Output = Result<AuthInfo, Self::Error>>
    where
        Self: Sized;

    fn validate(&self, req: &HttpRequest) -> Self::Future
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum AuthProvider {
    Htpasswd(HtpasswdAuth),
    None(NoneAuth),
}

impl CheckAuthentication for AuthProvider {
    type Error = crate::error::Error;
    type Future = Ready<Result<AuthInfo, Self::Error>>;

    fn validate(&self, req: &HttpRequest) -> Self::Future
    where
        Self: Sized,
    {
        match self {
            Self::Htpasswd(auth) => auth.validate(req),
            Self::None(auth) => auth.validate(req),
        }
    }
}
