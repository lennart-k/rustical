use super::methods::mkcol::route_mkcol;
use super::methods::report::route_report_addressbook;
use crate::address_object::AddressObjectResourceService;
use crate::address_object::resource::AddressObjectResource;
use crate::addressbook::methods::get::route_get;
use crate::addressbook::methods::post::route_post;
use crate::addressbook::methods::put::route_put;
use crate::addressbook::resource::AddressbookResource;
use crate::{CardDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::Response;
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::auth::Principal;
use rustical_store::{AddressbookStore, SubscriptionStore};
use std::convert::Infallible;
use std::sync::Arc;
use tower::Service;

pub struct AddressbookResourceService<AS: AddressbookStore, S: SubscriptionStore> {
    pub(crate) addr_store: Arc<AS>,
    pub(crate) sub_store: Arc<S>,
}

impl<A: AddressbookStore, S: SubscriptionStore> AddressbookResourceService<A, S> {
    pub fn new(addr_store: Arc<A>, sub_store: Arc<S>) -> Self {
        Self {
            addr_store,
            sub_store,
        }
    }
}

impl<A: AddressbookStore, S: SubscriptionStore> Clone for AddressbookResourceService<A, S> {
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            sub_store: self.sub_store.clone(),
        }
    }
}

#[async_trait]
impl<AS: AddressbookStore, S: SubscriptionStore> ResourceService
    for AddressbookResourceService<AS, S>
{
    type MemberType = AddressObjectResource;
    type PathComponents = (String, String); // principal, addressbook_id
    type Resource = AddressbookResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook, webdav-push";

    async fn get_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Self::Resource, Error> {
        let addressbook = self
            .addr_store
            .get_addressbook(principal, addressbook_id, false)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(addressbook.into())
    }

    async fn get_members(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        Ok(self
            .addr_store
            .get_objects(principal, addressbook_id)
            .await?
            .into_iter()
            .map(|object| AddressObjectResource {
                object,
                principal: principal.to_owned(),
            })
            .collect())
    }

    async fn save_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        self.addr_store
            .update_addressbook(principal.to_owned(), addressbook_id.to_owned(), file.into())
            .await?;
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.addr_store
            .delete_addressbook(principal, addressbook_id, use_trashbin)
            .await?;
        Ok(())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> Router<State> {
        Router::new()
            .nest(
                "/{object_id}",
                AddressObjectResourceService::new(self.addr_store.clone()).axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<AS: AddressbookStore, S: SubscriptionStore> AxumMethods for AddressbookResourceService<AS, S> {
    fn report() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_report_addressbook::<AS, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn get() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_get::<AS, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn post() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_post::<AS, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn put() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_put::<AS, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn mkcol() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcol::<AS, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
}
