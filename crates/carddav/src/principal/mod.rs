use crate::Error;
use rustical_dav::extensions::CommonPropertiesExtension;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::xml::{
    GroupMemberSet, GroupMembership, HrefElement, Resourcetype, ResourcetypeInner,
};
use rustical_store::auth::Principal;

mod service;
pub use service::*;
mod prop;
pub use prop::*;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    pub principal: Principal,
    pub members: Vec<String>,
}

impl ResourceName for PrincipalResource {
    fn get_name(&self) -> String {
        self.principal.id.clone()
    }
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type Principal = Principal;

    fn is_collection(&self) -> bool {
        true
    }

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "principal"),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(puri.principal_uri(&self.principal.id));

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
                    PrincipalPropName::AddressbookHomeSet => {
                        PrincipalProp::AddressbookHomeSet(AddressbookHomeSet(
                            self.principal
                                .memberships()
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::PrincipalAddress => PrincipalProp::PrincipalAddress(None),
                    PrincipalPropName::GroupMembership => {
                        PrincipalProp::GroupMembership(GroupMembership(
                            self.principal
                                .memberships_without_self()
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::GroupMemberSet => {
                        PrincipalProp::GroupMemberSet(GroupMemberSet(
                            self.members
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::AlternateUriSet => PrincipalProp::AlternateUriSet,
                    PrincipalPropName::PrincipalCollectionSet => {
                        PrincipalProp::PrincipalCollectionSet(puri.principal_collection().into())
                    }
                })
            }

            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn get_displayname(&self) -> Option<&str> {
        Some(
            self.principal
                .displayname
                .as_ref()
                .unwrap_or(&self.principal.id),
        )
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal.id)
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal.id),
        ))
    }
}
