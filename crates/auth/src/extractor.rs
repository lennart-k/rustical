use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use futures_util::Future;
use std::{marker::PhantomData, task::Poll};

use crate::error::Error;

use super::{AuthInfo, CheckAuthentication};

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

pub struct AuthInfoExtractorFuture<A: CheckAuthentication>(Result<AuthInfo, Error>, PhantomData<A>);

impl<A: CheckAuthentication> Future for AuthInfoExtractorFuture<A> {
    type Output = Result<AuthInfoExtractor<A>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.0 {
            Ok(auth_info) => Poll::Ready(Ok(AuthInfoExtractor {
                inner: auth_info.clone(),
                _provider_type: PhantomData,
            })),
            Err(err) => Poll::Ready(Err(err.clone())),
        }
    }
}

impl<A> FromRequest for AuthInfoExtractor<A>
where
    A: CheckAuthentication,
{
    type Error = Error;
    type Future = AuthInfoExtractorFuture<A>;

    fn extract(req: &HttpRequest) -> Self::Future {
        let result = req.app_data::<Data<A>>().unwrap().validate(req);
        AuthInfoExtractorFuture(result, PhantomData)
    }
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Self::extract(req)
    }
}
