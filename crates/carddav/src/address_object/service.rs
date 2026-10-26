use super::methods::{get_object, put_object};
use crate::{CardDavPrincipalUri, Error, address_object::resource::AddressObjectResource};
use async_trait::async_trait;
use axum::{extract::Request, handler::Handler, response::Response};
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::access::{self, Resolved};
use rustical_store::{AddressbookStore, Preconditions, ShareStore, auth::Principal, shared};
use serde::{Deserialize, Deserializer};
use std::{convert::Infallible, sync::Arc};
use tower::Service;

pub struct AddressObjectResourceService<AS: AddressbookStore> {
    pub(crate) addr_store: Arc<AS>,
    pub(crate) share_store: Arc<dyn ShareStore>,
}

impl<AS: AddressbookStore> AddressObjectResourceService<AS> {
    #[must_use]
    pub const fn new(addr_store: Arc<AS>) -> Self {
        Self { addr_store }
    }
}

impl<AS: AddressbookStore> Clone for AddressObjectResourceService<AS> {
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            share_store: self.share_store.clone(),
        }
    }
}

impl<AS: AddressbookStore> AddressObjectResourceService<AS> {
    /// The addressbook of an object path
    pub(crate) async fn addressbook(
        &self,
        principal: &str,
        addressbook_id: &str,
        show_deleted: bool,
    ) -> Result<Resolved, Error> {
        let (resolved, _addressbook) = access::resolve_addressbook(
            self.share_store.as_ref(),
            self.addr_store.as_ref(),
            principal,
            addressbook_id,
            show_deleted,
        )
        .await?;
        Ok(resolved)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddressObjectPathComponents {
    pub principal: String,
    pub addressbook_id: String,
    #[serde(deserialize_with = "deserialize_vcf_name")]
    pub object_id: String,
}

#[async_trait]
impl<AS: AddressbookStore> ResourceService for AddressObjectResourceService<AS> {
    type PathComponents = AddressObjectPathComponents;
    type Resource = AddressObjectResource;
    type MemberType = AddressObjectResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook";

    async fn get_resource(
        &self,
        AddressObjectPathComponents {
            principal,
            addressbook_id,
            object_id,
        }: &Self::PathComponents,
        show_deleted: bool,
    ) -> Result<Self::Resource, Self::Error> {
        let resolved = self.addressbook(principal, addressbook_id, true).await?;
        let object =
            shared::address_object(self.addr_store.as_ref(), &resolved, object_id, show_deleted)
                .await?;
        Ok(AddressObjectResource {
            object,
            object_id: object_id.to_owned(),
            principal: principal.to_owned(),
            resolved,
        })
    }

    async fn delete_resource(
        &self,
        AddressObjectPathComponents {
            principal,
            addressbook_id,
            object_id,
        }: &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        let resolved = self.addressbook(principal, addressbook_id, true).await?;
        // The generic DELETE route checked the preconditions
        shared::delete_address_object(
            self.addr_store.as_ref(),
            self.share_store.as_ref(),
            &resolved,
            object_id,
            use_trashbin,
            Preconditions::default(),
        )
        .await?;
        Ok(())
    }
}

impl<AS: AddressbookStore> AxumMethods for AddressObjectResourceService<AS> {
    fn get() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(get_object::<AS>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn put() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(put_object::<AS>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
}

fn deserialize_vcf_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name: String = Deserialize::deserialize(deserializer)?;
    name.strip_suffix(".vcf").map_or_else(
        || Err(serde::de::Error::custom("Missing .vcf extension")),
        |object_id| Ok(object_id.to_owned()),
    )
}
