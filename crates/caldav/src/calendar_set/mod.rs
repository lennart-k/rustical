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
pub struct CalendarSetResource {
    pub(crate) principal: String,
    pub(crate) read_only: bool,
    pub(crate) name: &'static str,
}

impl ResourceName for CalendarSetResource {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }
}

impl Resource for CalendarSetResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type Principal = User;

    const IS_COLLECTION: bool = true;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(
            Some(rustical_dav::namespace::NS_DAV),
            "collection",
        )])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                <Self as CommonPropertiesExtension>::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(if self.read_only {
            UserPrivilegeSet::owner_read(user.is_principal(&self.principal))
        } else {
            UserPrivilegeSet::owner_only(user.is_principal(&self.principal))
        })
    }
}
