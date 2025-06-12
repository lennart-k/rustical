use crate::Error;
use rustical_dav::extensions::CommonPropertiesExtension;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::xml::{Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;

mod service;
pub use service::*;
mod prop;
pub use prop::*;

#[derive(Clone)]
pub struct PrincipalResource {
    principal: User,
}

impl ResourceName for PrincipalResource {
    fn get_name(&self) -> String {
        self.principal.id.to_owned()
    }
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type Principal = User;

    const IS_COLLECTION: bool = true;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "principal"),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_url = puri.principal_uri(&self.principal.id);

        let home_set = CalendarHomeSet(
            user.memberships()
                .into_iter()
                .map(|principal| puri.principal_uri(principal).into())
                .collect(),
        );

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::CalendarUserType => {
                        PrincipalProp::CalendarUserType(self.principal.principal_type.to_owned())
                    }
                    PrincipalPropName::PrincipalUrl => {
                        PrincipalProp::PrincipalUrl(principal_url.into())
                    }
                    PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(home_set),
                    PrincipalPropName::CalendarUserAddressSet => {
                        PrincipalProp::CalendarUserAddressSet(principal_url.into())
                    }
                    PrincipalPropName::GroupMembership => {
                        PrincipalProp::GroupMembership(GroupMembership(
                            user.memberships_without_self()
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::AlternateUriSet => PrincipalProp::AlternateUriSet,
                })
            }
            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                <Self as CommonPropertiesExtension>::get_prop(self, puri, user, prop)?,
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

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_read(
            user.is_principal(&self.principal.id),
        ))
    }
}
