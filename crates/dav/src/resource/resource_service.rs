use super::methods::handle_propfind;
use super::Resource;
use actix_web::error::UrlGenerationError;
use actix_web::test::TestRequest;
use actix_web::{dev::ResourceMap, ResponseError};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{OptionalFromRequestParts, Path};
use axum::handler::Handler;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tower::util::BoxCloneSyncService;
use tower::Service;

#[async_trait]
pub trait ResourceService: Sized + Send + Sync + Clone + 'static {
    type MemberType: Resource<Error = Self::Error>;
    type PathComponents: DeserializeOwned + Sized + Send + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error>;
    type Error: ResponseError + From<crate::Error> + IntoResponse;

    async fn get_members(
        &self,
        _path_components: &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![])
    }

    async fn get_resource(
        &self,
        _path: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error>;
    async fn save_resource(
        &self,
        _path: &Self::PathComponents,
        _file: Self::Resource,
    ) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }
    async fn delete_resource(
        &self,
        _path: &Self::PathComponents,
        _use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }

    /// Hook for other resources to insert their additional methods (i.e. REPORT, MKCALENDAR)
    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res
    }

    #[inline]
    fn axum_service(self) -> ResourceServiceRouter<Self>
    where
        Self: Send + Sync + Clone,
    {
        self.into()
    }
}

#[derive(Clone)]
pub struct ResourceServiceRouter<RS: ResourceService> {
    state: Arc<RS>,
    propfind_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // proppatch_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // delete_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    fallback_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
}

impl<RS: ResourceService + Clone> From<RS> for ResourceServiceRouter<RS> {
    fn from(state: RS) -> Self {
        let state = Arc::new(state);
        let propfind_srv =
            BoxCloneSyncService::new(Handler::with_state(handle_propfind::<RS>, state.clone()));
        let fallback_srv =
            BoxCloneSyncService::new(Handler::with_state(handle_fallback, state.clone()));
        ResourceServiceRouter {
            state,
            propfind_srv,
            fallback_srv,
        }
    }
}

impl<RS: ResourceService> Service<Request<Body>> for ResourceServiceRouter<RS>
where
    RS: Send + Sync + Clone,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<
        Box<
            (dyn futures_util::Future<Output = Result<Response<axum::body::Body>, Infallible>>
                 + std::marker::Send
                 + 'static),
        >,
    >;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // let propfind_srv = BoxCloneSyncService::new(Handler::with_state(
        //     handle_propfind::<RS>,
        //     self.state.clone(),
        // ));
        // let fallback_srv =
        //     BoxCloneSyncService::new(Handler::with_state(handle_fallback, self.state.clone()));
        match req.method().as_str() {
            "PROPFIND" => Service::call(&mut self.propfind_srv.clone(), req),
            _ => Service::call(&mut self.fallback_srv.clone(), req),
        }
    }
}

async fn handle_fallback() -> impl IntoResponse {
    "FALLBACK"
}

pub trait NamedRoute {
    fn route_name() -> &'static str;

    fn get_url<U, I>(rmap: &ResourceMap, elements: U) -> Result<String, UrlGenerationError>
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        Ok(rmap
            .url_for(
                &TestRequest::default().to_http_request(),
                Self::route_name(),
                elements,
            )?
            .path()
            .to_owned())
    }
}
