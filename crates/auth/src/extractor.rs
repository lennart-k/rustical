use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use std::{
    future::{ready, Ready},
    marker::PhantomData,
};

use crate::error::Error;

use super::{AuthInfo, CheckAuthentication};

#[derive(Clone)]
pub struct AuthInfoExtractor<A: CheckAuthentication> {
    pub inner: AuthInfo,
    pub _provider_type: PhantomData<A>,
}

impl<T: CheckAuthentication> From<AuthInfo> for AuthInfoExtractor<T> {
    fn from(value: AuthInfo) -> Self {
        AuthInfoExtractor {
            inner: value,
            _provider_type: PhantomData::<T>,
        }
    }
}

impl<A> FromRequest for AuthInfoExtractor<A>
where
    A: CheckAuthentication,
{
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = req.app_data::<Data<A>>().unwrap().validate(req);
        ready(result.map(|auth_info| Self {
            inner: auth_info,
            _provider_type: PhantomData,
        }))
    }
}
