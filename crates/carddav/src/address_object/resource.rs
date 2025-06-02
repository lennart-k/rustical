use crate::{CardDavPrincipalUri, Error};
use async_trait::async_trait;
use derive_more::derive::{Constructor, From, Into};
use rustical_dav::{
    extensions::{CommonPropertiesExtension, CommonPropertiesProp},
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource, ResourceService},
    xml::Resourcetype,
};
use rustical_store::{AddressObject, AddressbookStore, auth::User};
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use serde::Deserialize;
use std::sync::Arc;

use super::methods::{get_object, put_object};

#[derive(Constructor)]
pub struct AddressObjectResourceService<AS: AddressbookStore> {
    addr_store: Arc<AS>,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
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

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct AddressObjectPathComponents {
    pub principal: String,
    pub addressbook_id: String,
    pub object_id: String,
}

#[async_trait(?Send)]
impl<AS: AddressbookStore> ResourceService for AddressObjectResourceService<AS> {
    type PathComponents = AddressObjectPathComponents;
    type Resource = AddressObjectResource;
    type MemberType = AddressObjectResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CardDavPrincipalUri;

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

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res.get(get_object::<AS>).put(put_object::<AS>)
    }
}
