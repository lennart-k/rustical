use crate::{CardDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::{extract::Request, handler::Handler, response::Response};
use derive_more::derive::{Constructor, From, Into};
use futures_util::future::BoxFuture;
use rustical_dav::{
    extensions::{CommonPropertiesExtension, CommonPropertiesProp},
    privileges::UserPrivilegeSet,
    resource::{AxumMethods, PrincipalUri, Resource, ResourceService},
    xml::Resourcetype,
};
use rustical_ical::AddressObject;
use rustical_store::{AddressbookStore, auth::User};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use serde::{Deserialize, Deserializer};
use std::{convert::Infallible, sync::Arc};
use tower::Service;

use super::methods::{get_object, put_object};

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

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressObjectPropName")]
pub enum AddressObjectProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Getetag(String),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressData(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressObjectPropWrapperName", untagged)]
pub enum AddressObjectPropWrapper {
    AddressObject(AddressObjectProp),
    Common(CommonPropertiesProp),
}

#[derive(Clone, From, Into)]
pub struct AddressObjectResource {
    pub object: AddressObject,
    pub principal: String,
}

impl Resource for AddressObjectResource {
    type Prop = AddressObjectPropWrapper;
    type Error = Error;
    type Principal = User;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &AddressObjectPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressObjectPropWrapperName::AddressObject(prop) => {
                AddressObjectPropWrapper::AddressObject(match prop {
                    AddressObjectPropName::Getetag => {
                        AddressObjectProp::Getetag(self.object.get_etag())
                    }
                    AddressObjectPropName::AddressData => {
                        AddressObjectProp::AddressData(self.object.get_vcf().to_owned())
                    }
                    AddressObjectPropName::Getcontenttype => {
                        AddressObjectProp::Getcontenttype("text/vcard;charset=utf-8")
                    }
                })
            }
            AddressObjectPropWrapperName::Common(prop) => AddressObjectPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal),
        ))
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
