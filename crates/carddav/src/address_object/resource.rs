use crate::{principal::PrincipalResource, Error};
use actix_web::{dev::ResourceMap, web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::{
    privileges::UserPrivilegeSet,
    resource::{InvalidProperty, Resource, ResourceService},
    xml::HrefElement,
};
use rustical_store::{auth::User, AddressObject, AddressbookStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};

use super::methods::{get_object, put_object};

pub struct AddressObjectResourceService<AS: AddressbookStore + ?Sized> {
    pub addr_store: Arc<AS>,
    pub path: String,
    pub principal: String,
    pub cal_id: String,
    pub object_id: String,
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum AddressObjectPropName {
    Getetag,
    AddressData,
    Getcontenttype,
    CurrentUserPrincipal,
    Owner,
    CurrentUserPrivilegeSet,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum AddressObjectProp {
    // WebDAV (RFC 2518)
    Getetag(String),
    Getcontenttype(String),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control (RFC 3744)
    Owner(HrefElement),
    CurrentUserPrivilegeSet(UserPrivilegeSet),

    // CalDAV (RFC 4791)
    #[serde(rename = "CARD:address-data")]
    AddressData(String),
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for AddressObjectProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, From, Into)]
pub struct AddressObjectResource {
    pub object: AddressObject,
    pub principal: String,
}

impl Resource for AddressObjectResource {
    type PropName = AddressObjectPropName;
    type Prop = AddressObjectProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressObjectPropName::Getetag => AddressObjectProp::Getetag(self.object.get_etag()),
            AddressObjectPropName::AddressData => {
                AddressObjectProp::AddressData(self.object.get_vcf().to_owned())
            }
            AddressObjectPropName::Getcontenttype => {
                AddressObjectProp::Getcontenttype("text/vcard;charset=utf-8".to_owned())
            }
            AddressObjectPropName::CurrentUserPrincipal => AddressObjectProp::CurrentUserPrincipal(
                HrefElement::new(PrincipalResource::get_principal_url(rmap, &user.id)),
            ),
            AddressObjectPropName::Owner => AddressObjectProp::Owner(
                PrincipalResource::get_principal_url(rmap, &self.principal).into(),
            ),
            AddressObjectPropName::CurrentUserPrivilegeSet => {
                AddressObjectProp::CurrentUserPrivilegeSet(UserPrivilegeSet::all())
            }
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "carddav_address_object"
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

#[derive(Debug, Clone)]
pub struct AddressObjectPathComponents {
    pub principal: String,
    pub cal_id: String,
    pub object_id: String,
}

impl<'de> Deserialize<'de> for AddressObjectPathComponents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        type Inner = (String, String, String);
        let (principal, calendar, mut object) = Inner::deserialize(deserializer)?;
        if object.ends_with(".vcf") {
            object.truncate(object.len() - 4);
        }
        Ok(Self {
            principal,
            cal_id: calendar,
            object_id: object,
        })
    }
}

#[async_trait(?Send)]
impl<AS: AddressbookStore + ?Sized> ResourceService for AddressObjectResourceService<AS> {
    type PathComponents = AddressObjectPathComponents;
    type Resource = AddressObjectResource;
    type MemberType = AddressObjectResource;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let AddressObjectPathComponents {
            principal,
            cal_id,
            object_id,
        } = path_components;

        let addr_store = req
            .app_data::<Data<AS>>()
            .expect("no addressbook store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            addr_store,
            principal,
            cal_id,
            object_id,
            path: req.path().to_string(),
        })
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        let object = self
            .addr_store
            .get_object(&self.principal, &self.cal_id, &self.object_id)
            .await?;
        Ok(AddressObjectResource {
            object,
            principal: self.principal.to_owned(),
        })
    }

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }

    async fn delete_resource(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.addr_store
            .delete_object(&self.principal, &self.cal_id, &self.object_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res.get(get_object::<AS>).put(put_object::<AS>)
    }
}
