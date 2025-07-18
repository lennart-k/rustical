use crate::Error;
use rustical_dav::extensions::CommonPropertiesExtension;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::xml::{
    GroupMemberSet, GroupMembership, Resourcetype, ResourcetypeInner, SupportedReportSet,
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
    principal: Principal,
    members: Vec<String>,
    // If true only return the principal as the calendar home set, otherwise also groups
    simplified_home_set: bool,
}

impl ResourceName for PrincipalResource {
    fn get_name(&self) -> String {
        self.principal.id.to_owned()
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
            // https://github.com/apple/ccs-calendarserver/blob/13c706b985fb728b9aab42dc0fef85aae21921c3/doc/Extensions/caldav-proxy.txt
            // ResourcetypeInner(
            //     Some(rustical_dav::namespace::NS_CALENDARSERVER),
            //     "calendar-proxy-write",
            // ),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_url = puri.principal_uri(&self.principal.id);

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::CalendarUserType => {
                        PrincipalProp::CalendarUserType(self.principal.principal_type.to_owned())
                    }
                    PrincipalPropName::PrincipalUrl => {
                        PrincipalProp::PrincipalUrl(principal_url.into())
                    }
                    PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(
                        CalendarHomeSet(if self.simplified_home_set {
                            vec![principal_url.into()]
                        } else {
                            self.principal
                                .memberships()
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect()
                        }),
                    ),
                    PrincipalPropName::CalendarUserAddressSet => {
                        PrincipalProp::CalendarUserAddressSet(principal_url.into())
                    }
                    PrincipalPropName::GroupMemberSet => {
                        PrincipalProp::GroupMemberSet(GroupMemberSet(
                            self.members
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::GroupMembership => {
                        PrincipalProp::GroupMembership(GroupMembership(
                            self.principal
                                .memberships_without_self()
                                .iter()
                                .map(|principal| puri.principal_uri(principal).into())
                                .collect(),
                        ))
                    }
                    PrincipalPropName::AlternateUriSet => PrincipalProp::AlternateUriSet,
                    // PrincipalPropName::PrincipalCollectionSet => {
                    //     PrincipalProp::PrincipalCollectionSet(puri.principal_collection().into())
                    // }
                    PrincipalPropName::SupportedReportSet => {
                        PrincipalProp::SupportedReportSet(SupportedReportSet::all())
                    }
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

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_read(
            user.is_principal(&self.principal.id),
        ))
    }
}
