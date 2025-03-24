use super::methods::handle_propfind;
use super::ResourceService;
use axum::body::Body;
use axum::handler::Handler;
use axum::http::Method;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use educe::Educe;
use futures_util::Future;
use rustical_store::auth::user::ToAuthenticationProvider;
use rustical_store::auth::AuthenticationProvider;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tower::util::BoxCloneSyncService;
use tower::Service;

pub struct ResourceServiceRouterState<AP: AuthenticationProvider, RS: ResourceService> {
    pub resource_service: Arc<RS>,
    pub auth_provider: Arc<AP>,
}

impl<AP: AuthenticationProvider, RS: ResourceService> Clone for ResourceServiceRouterState<AP, RS> {
    fn clone(&self) -> Self {
        Self {
            resource_service: self.resource_service.clone(),
            auth_provider: self.auth_provider.clone(),
        }
    }
}

impl<AP: AuthenticationProvider, RS: ResourceService> ToAuthenticationProvider
    for ResourceServiceRouterState<AP, RS>
{
    #[allow(refining_impl_trait)]
    fn auth_provider(&self) -> &AP {
        &self.auth_provider
    }
}

#[derive(Educe)]
#[educe(Clone)]
pub struct ResourceServiceRouter<RS: ResourceService + Clone, AP: AuthenticationProvider> {
    state: ResourceServiceRouterState<AP, RS>,
    propfind_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // proppatch_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // delete_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    fallback_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    mkcalendar_srv: Option<BoxCloneSyncService<Request<Body>, Response, Infallible>>,
    mkcol_srv: Option<BoxCloneSyncService<Request<Body>, Response, Infallible>>,
    report_srv: Option<BoxCloneSyncService<Request<Body>, Response, Infallible>>,
    get_srv: Option<BoxCloneSyncService<Request<Body>, Response, Infallible>>,
}

impl<RS: ResourceService + Clone, AP: AuthenticationProvider> ResourceServiceRouter<RS, AP> {
    pub fn new(resource_service: RS, auth_provider: Arc<AP>) -> Self {
        let state = ResourceServiceRouterState {
            resource_service: Arc::new(resource_service),
            auth_provider,
        };
        let propfind_srv = BoxCloneSyncService::new(Handler::with_state(
            handle_propfind::<AP, RS>,
            state.clone(),
        ));
        let fallback_srv =
            BoxCloneSyncService::new(Handler::with_state(handle_fallback, state.clone()));
        ResourceServiceRouter {
            state,
            propfind_srv,
            fallback_srv,
            mkcalendar_srv: None,
            mkcol_srv: None,
            report_srv: None,
            get_srv: None,
        }
    }

    pub fn report<T: 'static>(
        mut self,
        inner: impl Handler<T, ResourceServiceRouterState<AP, RS>>,
    ) -> Self {
        self.report_srv = Some(BoxCloneSyncService::new(Handler::with_state(
            inner,
            self.state.clone(),
        )));
        self
    }

    pub fn mkcalendar<T: 'static>(
        mut self,
        inner: impl Handler<T, ResourceServiceRouterState<AP, RS>>,
    ) -> Self {
        self.mkcalendar_srv = Some(BoxCloneSyncService::new(Handler::with_state(
            inner,
            self.state.clone(),
        )));
        self
    }
}

impl<RS: ResourceService + Clone, AP: AuthenticationProvider> Service<Request<Body>>
    for ResourceServiceRouter<RS, AP>
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<
        Box<
            (dyn Future<Output = Result<Response<axum::body::Body>, Infallible>>
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
        match req.method() {
            &Method::GET => {
                if let Some(srv) = &self.get_srv {
                    Service::call(&mut srv.clone(), req)
                } else {
                    Service::call(&mut self.fallback_srv.clone(), req)
                }
            }
            method => match method.as_str() {
                "PROPFIND" => Service::call(&mut self.propfind_srv.clone(), req),
                "MKCALENDAR" => {
                    if let Some(srv) = &self.mkcalendar_srv {
                        Service::call(&mut srv.clone(), req)
                    } else {
                        Service::call(&mut self.fallback_srv.clone(), req)
                    }
                }
                "MKCOL" => {
                    if let Some(srv) = &self.mkcol_srv {
                        Service::call(&mut srv.clone(), req)
                    } else {
                        Service::call(&mut self.fallback_srv.clone(), req)
                    }
                }
                "REPORT" => {
                    if let Some(srv) = &self.report_srv {
                        Service::call(&mut srv.clone(), req)
                    } else {
                        Service::call(&mut self.fallback_srv.clone(), req)
                    }
                }
                _ => Service::call(&mut self.fallback_srv.clone(), req),
            },
        }
    }
}

async fn handle_fallback() -> impl IntoResponse {
    "FALLBACK"
}
