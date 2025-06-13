use axum::{
    RequestExt,
    body::Body,
    extract::{Path, Request},
    response::{IntoResponse, Response},
};
use futures_core::future::BoxFuture;
use headers::{ContentType, ETag, HeaderMapExt};
use http::{Method, StatusCode};
use rust_embed::RustEmbed;
use std::{convert::Infallible, marker::PhantomData, str::FromStr};
use tower::Service;

#[derive(Clone, RustEmbed, Default)]
#[folder = "public/assets"]
pub struct Assets;

#[derive(Clone, Default)]
pub struct EmbedService<E>
where
    E: 'static + RustEmbed,
{
    _embed: PhantomData<E>,
}

impl<E> Service<Request> for EmbedService<E>
where
    E: 'static + RustEmbed,
{
    type Response = Response;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    #[inline]
    fn call(&mut self, mut req: Request) -> Self::Future {
        Box::pin(async move {
            if req.method() != Method::GET && req.method() != Method::HEAD {
                return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
            }
            let path: String = if let Ok(Path(path)) = req.extract_parts().await.unwrap() {
                path
            } else {
                return Ok(StatusCode::NOT_FOUND.into_response());
            };

            match E::get(&path) {
                Some(file) => {
                    let data = file.data;
                    let hash = hex::encode(file.metadata.sha256_hash());
                    let etag = format!("\"{hash}\"");
                    let mime = mime_guess::from_path(path).first_or_octet_stream();

                    let body = if req.method() == Method::HEAD {
                        Default::default()
                    } else {
                        data
                    };
                    let mut res = Response::builder().status(StatusCode::OK);
                    let hdrs = res.headers_mut().unwrap();
                    hdrs.typed_insert(ContentType::from(mime));
                    hdrs.typed_insert(ETag::from_str(&etag).unwrap());
                    Ok(res.body(Body::from(body)).unwrap())
                }
                None => Ok(StatusCode::NOT_FOUND.into_response()),
            }
        })
    }
}
