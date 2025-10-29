use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use headers::Allow;
use http::Method;
use std::{convert::Infallible, str::FromStr};

pub type MethodFunction<State> =
    fn(State, Request) -> BoxFuture<'static, Result<Response, Infallible>>;

pub trait AxumMethods: Sized + Send + Sync + 'static {
    #[inline]
    #[must_use]
    fn report() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn get() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn post() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn mkcol() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn mkcalendar() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn put() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn import() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    #[must_use]
    fn allow_header() -> Allow {
        let mut allow = vec![
            Method::from_str("PROPFIND").unwrap(),
            Method::from_str("PROPPATCH").unwrap(),
            Method::from_str("COPY").unwrap(),
            Method::from_str("MOVE").unwrap(),
            Method::DELETE,
            Method::OPTIONS,
        ];
        if Self::report().is_some() {
            allow.push(Method::from_str("REPORT").unwrap());
        }
        if Self::get().is_some() {
            allow.push(Method::GET);
            allow.push(Method::HEAD);
        }
        if Self::post().is_some() {
            allow.push(Method::POST);
        }
        if Self::mkcol().is_some() {
            allow.push(Method::from_str("MKCOL").unwrap());
        }
        if Self::mkcalendar().is_some() {
            allow.push(Method::from_str("MKCALENDAR").unwrap());
        }
        if Self::put().is_some() {
            allow.push(Method::PUT);
        }
        if Self::import().is_some() {
            allow.push(Method::from_str("IMPORT").unwrap());
        }

        allow.into_iter().collect()
    }
}
