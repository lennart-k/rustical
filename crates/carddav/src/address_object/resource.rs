use crate::{principal::PrincipalResource, Error};
use actix_web::{dev::ResourceMap, web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::{
    privileges::UserPrivilegeSet,
    resource::{Resource, ResourceService},
};
use rustical_store::{auth::User, AddressObject, AddressbookStore};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use serde::Deserialize;
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

use super::methods::{get_object, put_object};

pub struct AddressObjectResourceService<AS: AddressbookStore + ?Sized> {
    addr_store: Arc<AS>,
    principal: String,
    cal_id: String,
    object_id: String,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(AddressObjectPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
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

#[derive(Clone, From, Into)]
pub struct AddressObjectResource {
    pub object: AddressObject,
    pub principal: String,
}

impl Resource for AddressObjectResource {
    type PropName = AddressObjectPropName;
    type Prop = AddressObjectProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> &'static [&'static str] {
        &[]
    }

    fn get_prop(
        &self,
        _rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressObjectPropName::Getetag => AddressObjectProp::Getetag(self.object.get_etag()),
            AddressObjectPropName::AddressData => {
                AddressObjectProp::AddressData(self.object.get_vcf().to_owned())
            }
            AddressObjectPropName::Getcontenttype => {
                AddressObjectProp::Getcontenttype("text/vcard;charset=utf-8")
            }
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "carddav_address_object"
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
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
