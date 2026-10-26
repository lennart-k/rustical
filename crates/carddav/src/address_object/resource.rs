use std::borrow::Cow;

use crate::{
    Error,
    address_object::{
        AddressObjectProp, AddressObjectPropName, AddressObjectPropWrapper,
        AddressObjectPropWrapperName,
    },
};
use caldata::property::VcardFNProperty;
use rustical_dav::{
    extensions::CommonPropertiesExtension,
    privileges::{UserPrivilege, UserPrivilegeSet},
    resource::{PrincipalUri, Resource, ResourceName},
    resourcetype,
    xml::Resourcetype,
};
use rustical_ical::AddressObject;
use rustical_store::access::{self, Resolved, ResolvedKind};
use rustical_store::{Access, auth::Principal};

#[derive(Clone)]
pub struct AddressObjectResource {
    pub object: AddressObject,
    /// The home of the path
    pub principal: String,
    pub object_id: String,
    /// The addressbook the object belongs to
    pub resolved: Resolved,
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
        resourcetype!()
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
        Some(self.resolved.owner())
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        let may_use = access::may_use_home(user, &self.principal);
        Ok(match self.resolved.kind() {
            ResolvedKind::Own => UserPrivilegeSet::owner_only(may_use),
            ResolvedKind::Copy {
                access: Access::Write,
                ..
            } if may_use => UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                UserPrivilege::Write,
                UserPrivilege::WriteProperties,
                UserPrivilege::WriteContent,
            ]),
            ResolvedKind::Copy { .. } => UserPrivilegeSet::owner_read(may_use),
        })
    }
}
