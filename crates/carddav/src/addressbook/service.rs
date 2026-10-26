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
use rustical_store::auth::Principal;
use rustical_store::{AddressbookStore, ShareStore, ShareViewUpdate, access, shared};
use std::convert::Infallible;
use std::sync::Arc;
use tower::Service;

#[allow(clippy::struct_field_names)]
pub struct AddressbookResourceService<AS: AddressbookStore, DP: DavPushStore> {
    pub(crate) addr_store: Arc<AS>,
    pub(crate) share_store: Arc<dyn ShareStore>,
    pub(crate) dav_push_store: Arc<DP>,
}

impl<A: AddressbookStore, DP: DavPushStore> AddressbookResourceService<A, DP> {
    pub fn new(
        addr_store: Arc<A>,
        share_store: Arc<dyn ShareStore>,
        dav_push_store: Arc<DP>,
    ) -> Self {
        Self {
            addr_store,
            share_store,
            dav_push_store,
        }
    }

    /// The addressbook at the path as a resource
    pub(crate) async fn resource(
        &self,
        principal: &str,
        addressbook_id: &str,
        show_deleted: bool,
    ) -> Result<AddressbookResource, Error> {
        let (resolved, addressbook) = access::resolve_addressbook(
            self.share_store.as_ref(),
            self.addr_store.as_ref(),
            principal,
            addressbook_id,
            show_deleted,
        )
        .await?;
        let vapid_pubkey = self.dav_push_store.get_vapid_pubkey_b64().await?.clone();
        Ok(AddressbookResource {
            addressbook,
            vapid_pubkey,
            home: principal.to_owned(),
            name: addressbook_id.to_owned(),
            resolved,
        })
    }
}

impl<A: AddressbookStore, DP: DavPushStore> Clone for AddressbookResourceService<A, DP> {
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            share_store: self.share_store.clone(),
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
        self.resource(principal, addressbook_id, show_deleted)
            .await
            .map_err(|_e| Error::NotFound)
    }

    async fn get_members(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        let addressbook = self.resource(principal, addressbook_id, true).await?;
        Ok(
            shared::address_objects(self.addr_store.as_ref(), &addressbook.resolved)
                .await?
                .into_iter()
                .map(|(object_id, object)| AddressObjectResource {
                    object_id,
                    object,
                    principal: principal.to_owned(),
                    resolved: addressbook.resolved.clone(),
                })
                .collect(),
        )
    }

    async fn save_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        if file.resolved.is_own() {
            self.addr_store
                .update_addressbook(principal, addressbook_id, file.addressbook)
                .await?;
            return Ok(());
        }
        // A sharee changes only its own name for the share
        let before = self.resource(principal, addressbook_id, false).await?;
        if file.addressbook.displayname != before.addressbook.displayname {
            self.share_store
                .update_addressbook_share_view(
                    principal,
                    addressbook_id,
                    ShareViewUpdate {
                        displayname: Some(file.addressbook.displayname),
                        ..Default::default()
                    },
                )
                .await?;
        }
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        let addressbook = self.resource(principal, addressbook_id, true).await?;
        if addressbook.resolved.is_own() {
            self.addr_store
                .delete_addressbook(principal, addressbook_id, use_trashbin)
                .await?;
        } else {
            // Deleting a share only ends it for this home
            self.share_store
                .unshare_addressbook(
                    addressbook.resolved.owner(),
                    addressbook.resolved.id(),
                    principal,
                )
                .await?;
        }
        Ok(())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> Router<State> {
        Router::new()
            .nest(
                "/{object_id}",
                AddressObjectResourceService::new(
                    self.addr_store.clone(),
                    self.share_store.clone(),
                )
                .axum_router(),
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
