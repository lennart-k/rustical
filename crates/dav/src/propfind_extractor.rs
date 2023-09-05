use std::pin::Pin;

use actix_web::{http::StatusCode, Either, FromRequest, HttpRequest, ResponseError};
use derive_more::Display;
use futures_util::{
    future::{err, ok, Ready},
    Future,
};

#[derive(Debug, Display)]
pub struct BadPropfindRequest {}

impl ResponseError for BadPropfindRequest {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, PartialEq)]
pub struct Propfind(Vec<String>);

impl FromRequest for Propfind {
    type Error = BadPropfindRequest;
    type Future = Either<PropfindExtractFut, Ready<Result<Self, Self::Error>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {}
}

pub struct PropfindExtractFut {
    body_fut: HttpMessageBody,
}

impl Future for PropfindExtractFut {
    type Output = Result<Propfind, BadPropfindRequest>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new()
    }
}
