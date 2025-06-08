use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use headers::Allow;
use http::Method;
use std::{convert::Infallible, str::FromStr};

pub type MethodFunction<State> =
    fn(State, Request) -> BoxFuture<'static, Result<Response, Infallible>>;

pub trait AxumMethods: Sized + Send + Sync + 'static {
    #[inline]
    fn report() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn get() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn head() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn post() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn mkcol() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn mkcalendar() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn copy() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn mv() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn put() -> Option<MethodFunction<Self>> {
        None
    }

    #[inline]
    fn allow_header() -> Allow {
        let mut allow = vec![
            Method::from_str("PROPFIND").unwrap(),
            Method::from_str("PROPPATCH").unwrap(),
            Method::DELETE,
            Method::OPTIONS,
        ];
        if Self::report().is_some() {
            allow.push(Method::from_str("REPORT").unwrap());
        }
        if Self::get().is_some() {
            allow.push(Method::GET);
        }
        if Self::head().is_some() {
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
        if Self::copy().is_some() {
            allow.push(Method::from_str("COPY").unwrap());
        }
        if Self::mv().is_some() {
            allow.push(Method::from_str("MOVE").unwrap());
        }
        if Self::put().is_some() {
            allow.push(Method::PUT);
        }

        allow.into_iter().collect()
    }
}
