use std::marker::PhantomData;

use actix_web::{
    body::BoxBody,
    dev::{
        HttpServiceFactory, ResourceDef, Service, ServiceFactory, ServiceRequest, ServiceResponse,
    },
    http::{header, Method},
    HttpResponse,
};
use futures_core::future::LocalBoxFuture;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist/assets"]
pub struct Assets;

pub struct EmbedService<E>
where
    E: 'static + RustEmbed,
{
    _embed: PhantomData<E>,
    prefix: String,
}

impl<E> EmbedService<E>
where
    E: 'static + RustEmbed,
{
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            _embed: PhantomData,
        }
    }
}

impl<E> HttpServiceFactory for EmbedService<E>
where
    E: 'static + RustEmbed,
{
    fn register(self, config: &mut actix_web::dev::AppService) {
        let resource_def = if config.is_root() {
            ResourceDef::root_prefix(&self.prefix)
        } else {
            ResourceDef::prefix(&self.prefix)
        };
        config.register_service(resource_def, None, self, None);
    }
}

impl<E> ServiceFactory<ServiceRequest> for EmbedService<E>
where
    E: 'static + RustEmbed,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Config = ();
    type Service = EmbedService<E>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let prefix = self.prefix.clone();
        Box::pin(async move {
            Ok(Self {
                prefix,
                _embed: PhantomData,
            })
        })
    }
}

impl<E> Service<ServiceRequest> for EmbedService<E>
where
    E: 'static + RustEmbed,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::always_ready!();

    fn call(&self, req: ServiceRequest) -> Self::Future {
        Box::pin(async move {
            if req.method() != Method::GET && req.method() != Method::HEAD {
                return Ok(req.into_response(HttpResponse::MethodNotAllowed()));
            }
            let path = req.match_info().unprocessed().trim_start_matches('/');

            match E::get(path) {
                Some(file) => {
                    let data = file.data;
                    let hash = hex::encode(file.metadata.sha256_hash());
                    let mime = mime_guess::from_path(path).first_or_octet_stream();

                    let body = if req.method() == Method::HEAD {
                        Default::default()
                    } else {
                        data
                    };
                    Ok(req.into_response(
                        HttpResponse::Ok()
                            .content_type(mime)
                            .insert_header((header::ETAG, hash))
                            .body(body),
                    ))
                }
                None => Ok(req.into_response(HttpResponse::NotFound())),
            }
        })
    }
}
