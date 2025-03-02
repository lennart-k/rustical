use super::methods::handle_propfind;
use super::ResourceService;
use axum::body::Body;
use axum::handler::Handler;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
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
    fn auth_provider(&self) -> &AP {
        &self.auth_provider
    }
}

#[derive(Clone)]
pub struct ResourceServiceRouter {
    propfind_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // proppatch_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    // delete_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    fallback_srv: BoxCloneSyncService<Request<Body>, Response, Infallible>,
}

impl ResourceServiceRouter {
    pub fn new<RS: ResourceService + Clone, AP: AuthenticationProvider>(
        resource_service: RS,
        auth_provider: Arc<AP>,
    ) -> Self {
        let state = ResourceServiceRouterState {
            resource_service: Arc::new(resource_service),
            auth_provider,
        };
        let propfind_srv = BoxCloneSyncService::new(Handler::with_state(
            handle_propfind::<AP, RS>,
            state.clone(),
        ));
        let fallback_srv = BoxCloneSyncService::new(Handler::with_state(handle_fallback, state));
        ResourceServiceRouter {
            propfind_srv,
            fallback_srv,
        }
    }
}

impl Service<Request<Body>> for ResourceServiceRouter {
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
