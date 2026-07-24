use super::methods::mkcol::route_mkcol;
use super::methods::report::route_report_addressbook;
use crate::address_object::AddressObjectResourceService;
use crate::address_object::resource::AddressObjectResource;
use crate::addressbook::methods::get::route_get;
use crate::addressbook::methods::import::route_import;
use crate::addressbook::methods::post::route_post;
use crate::addressbook::resource::AddressbookResource;
use crate::{CardDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::Response;
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_dav_push::DavPushStore;
use rustical_store::AddressbookStore;
use rustical_store::auth::Principal;
use std::convert::Infallible;
use std::sync::Arc;
use tower::Service;

pub struct AddressbookResourceService<AS: AddressbookStore, DP: DavPushStore> {
    pub(crate) addr_store: Arc<AS>,
    pub(crate) dav_push_store: Arc<DP>,
}

impl<A: AddressbookStore, DP: DavPushStore> AddressbookResourceService<A, DP> {
    pub const fn new(addr_store: Arc<A>, dav_push_store: Arc<DP>) -> Self {
        Self {
            addr_store,
            dav_push_store,
        }
    }
}

impl<A: AddressbookStore, DP: DavPushStore> Clone for AddressbookResourceService<A, DP> {
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            dav_push_store: self.dav_push_store.clone(),
        }
    }
}

#[async_trait]
impl<AS: AddressbookStore, DP: DavPushStore> ResourceService
    for AddressbookResourceService<AS, DP>
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
        show_deleted: bool,
    ) -> Result<Self::Resource, Error> {
        let addressbook = self
            .addr_store
            .get_addressbook(principal, addressbook_id, show_deleted)
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
            .map(|(object_id, object)| AddressObjectResource {
                object_id,
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
            .update_addressbook(principal, addressbook_id, file.into())
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

impl<AS: AddressbookStore, DP: DavPushStore> AxumMethods for AddressbookResourceService<AS, DP> {
    fn report() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_report_addressbook::<AS, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn get() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_get::<AS, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn post() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_post::<AS, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn import() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_import::<AS, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn mkcol() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcol::<AS, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
}
