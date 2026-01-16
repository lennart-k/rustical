use std::borrow::Cow;

use crate::{
    Error,
    address_object::{
        AddressObjectProp, AddressObjectPropName, AddressObjectPropWrapper,
        AddressObjectPropWrapperName,
    },
};
use derive_more::derive::{From, Into};
use ical::parser::VcardFNProperty;
use rustical_dav::{
    extensions::CommonPropertiesExtension,
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource, ResourceName},
    xml::Resourcetype,
};
use rustical_ical::AddressObject;
use rustical_store::auth::Principal;

#[derive(Clone, From, Into)]
pub struct AddressObjectResource {
    pub object: AddressObject,
    pub principal: String,
    pub object_id: String,
}

impl ResourceName for AddressObjectResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(format!("{}.vcf", self.object_id))
    }
}

impl Resource for AddressObjectResource {
    type Prop = AddressObjectPropWrapper;
    type Error = Error;
    type Principal = Principal;

    fn is_collection(&self) -> bool {
        false
    }

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
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

    fn get_displayname(&self) -> Option<&str> {
        self.object
            .get_vcard()
            .full_name
            .first()
            .map(|VcardFNProperty(name, _)| name.as_str())
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal),
        ))
    }
}
