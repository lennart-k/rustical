use super::methods::{axum_route_propfind, axum_route_proppatch};
use crate::resource::{
    ResourceService,
    axum_methods::AxumMethods,
    methods::{axum_route_copy, axum_route_move},
};
use axum::{
    body::Body,
    extract::FromRequestParts,
    handler::Handler,
    http::{Request, Response},
    response::IntoResponse,
};
use futures_util::future::BoxFuture;
use headers::HeaderMapExt;
use http::{HeaderValue, StatusCode};
use std::convert::Infallible;
use tower::Service;

#[derive(Clone)]
pub struct AxumService<RS: ResourceService + AxumMethods> {
    resource_service: RS,
}

impl<RS: ResourceService + AxumMethods> AxumService<RS> {
    pub fn new(resource_service: RS) -> Self {
        Self { resource_service }
    }
}

impl<RS: ResourceService + AxumMethods + Clone + Send + Sync> Service<Request<Body>>
    for AxumService<RS>
where
    RS::Error: IntoResponse + Send + Sync + 'static,
    RS::Principal: FromRequestParts<RS>,
{
    type Error = Infallible;
    type Response = Response<Body>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    #[inline]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        use crate::resource::methods::axum_route_delete;
        let mut propfind_service =
            Handler::with_state(axum_route_propfind::<RS>, self.resource_service.clone());
        let mut proppatch_service =
            Handler::with_state(axum_route_proppatch::<RS>, self.resource_service.clone());
        let mut delete_service =
            Handler::with_state(axum_route_delete::<RS>, self.resource_service.clone());
        let mut move_service =
            Handler::with_state(axum_route_move::<RS>, self.resource_service.clone());
        let mut copy_service =
            Handler::with_state(axum_route_copy::<RS>, self.resource_service.clone());
        let mut options_service = Handler::with_state(route_options::<RS>, ());
        match req.method().as_str() {
            "PROPFIND" => return Box::pin(Service::call(&mut propfind_service, req)),
            "PROPPATCH" => return Box::pin(Service::call(&mut proppatch_service, req)),
            "DELETE" => return Box::pin(Service::call(&mut delete_service, req)),
            "OPTIONS" => return Box::pin(Service::call(&mut options_service, req)),
            "MOVE" => return Box::pin(Service::call(&mut move_service, req)),
            "COPY" => return Box::pin(Service::call(&mut copy_service, req)),
            "REPORT" => {
                if let Some(svc) = RS::report() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "GET" => {
                if let Some(svc) = RS::get() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "HEAD" => {
                if let Some(svc) = RS::head() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "POST" => {
                if let Some(svc) = RS::post() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "MKCOL" => {
                if let Some(svc) = RS::mkcol() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "MKCALENDAR" => {
                if let Some(svc) = RS::mkcalendar() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            "PUT" => {
                if let Some(svc) = RS::put() {
                    return svc(self.resource_service.clone(), req);
                }
            }
            _ => {}
        };
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Method not allowed"))
                .unwrap())
        })
    }
}

async fn route_options<RS: ResourceService + AxumMethods>() -> Response<Body> {
    let mut resp = Response::builder().status(StatusCode::OK);
    let headers = resp.headers_mut().unwrap();
    headers.insert("DAV", HeaderValue::from_static(RS::DAV_HEADER));
    headers.typed_insert(RS::allow_header());
    resp.body(Body::empty()).unwrap()
}
