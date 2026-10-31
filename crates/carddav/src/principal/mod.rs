use crate::Error;
use rustical_dav::extensions::CommonPropertiesExtension;
use rustical_dav::namespace::NS_DAV;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::resourcetype;
use rustical_dav::xml::{GroupMemberSet, GroupMembership, HrefElement, Resourcetype};
use rustical_store::{access, auth::Principal};
use std::borrow::Cow;

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
    /// Whom this principal shares addressbooks with: they see the principal, but only its name
    pub sharees: Vec<String>,
}

impl ResourceName for PrincipalResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(&self.principal.id)
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
        resourcetype!((NS_DAV, "collection"), (NS_DAV, "principal"))
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(puri.principal_uri(&self.principal.id));
        // A sharee learns who the owner is, not its homes, groups or members
        let full = access::may_use_home(user, &self.principal.id);

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
                    PrincipalPropName::AddressbookHomeSet => {
                        PrincipalProp::AddressbookHomeSet(AddressbookHomeSet(
                            self.principal
                                .memberships()
                                .iter()
                                .filter(|_| full)
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
                                .filter(|_| full)
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::GroupMemberSet => {
                        PrincipalProp::GroupMemberSet(GroupMemberSet(
                            self.members
                                .iter()
                                .filter(|_| full)
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
        if access::may_use_home(user, &self.principal.id) {
            Ok(UserPrivilegeSet::all())
        } else {
            Ok(UserPrivilegeSet::owner_read(access::may_see_principal(
                user,
                &self.principal.id,
                &self.sharees,
            )))
        }
    }
}
