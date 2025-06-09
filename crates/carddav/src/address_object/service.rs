use super::methods::{get_object, put_object};
use crate::{CardDavPrincipalUri, Error, address_object::resource::AddressObjectResource};
use async_trait::async_trait;
use axum::{extract::Request, handler::Handler, response::Response};
use derive_more::derive::Constructor;
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::{AddressbookStore, auth::User};
use serde::{Deserialize, Deserializer};
use std::{convert::Infallible, sync::Arc};
use tower::Service;

#[derive(Constructor)]
pub struct AddressObjectResourceService<AS: AddressbookStore> {
    pub(crate) addr_store: Arc<AS>,
}

impl<AS: AddressbookStore> Clone for AddressObjectResourceService<AS> {
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
        }
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
    type Principal = User;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook";
    const IS_COLLECTION: bool = false;

    async fn get_resource(
        &self,
        AddressObjectPathComponents {
            principal,
            addressbook_id,
            object_id,
        }: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let object = self
            .addr_store
            .get_object(principal, addressbook_id, object_id, false)
            .await?;
        Ok(AddressObjectResource {
            object,
            principal: principal.to_owned(),
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
        self.addr_store
            .delete_object(principal, addressbook_id, object_id, use_trashbin)
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
    if let Some(object_id) = name.strip_suffix(".vcf") {
        Ok(object_id.to_owned())
    } else {
        Err(serde::de::Error::custom("Missing .vcf extension"))
    }
}
