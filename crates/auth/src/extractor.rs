use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use futures_util::{Future, FutureExt};
use std::marker::PhantomData;
use std::pin::Pin;

use super::{CheckAuthentication, AuthInfo};

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

pub struct AuthInfoExtractorFuture<A>
where
    A: CheckAuthentication,
{
    future: Pin<Box<A::Future>>,
}

impl<A> Future for AuthInfoExtractorFuture<A>
where
    A: CheckAuthentication,
{
    type Output = Result<AuthInfoExtractor<A>, A::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut().future.poll_unpin(cx) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(result) => {
                std::task::Poll::Ready(result.map(|auth_info| auth_info.into()))
            }
        }
    }
}

impl<A> FromRequest for AuthInfoExtractor<A>
where
    A: CheckAuthentication,
{
    type Error = A::Error;
    type Future = AuthInfoExtractorFuture<A>;

    fn extract(req: &HttpRequest) -> Self::Future {
        let a = req.app_data::<Data<A>>().unwrap().validate(req);
        Self::Future {
            future: Box::pin(a),
        }
    }
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Self::extract(req)
    }
}
