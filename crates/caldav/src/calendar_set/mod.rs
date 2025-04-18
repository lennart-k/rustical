use crate::Error;
use crate::calendar::resource::CalendarResource;
use crate::principal::PrincipalResource;
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{Resourcetype, ResourcetypeInner};
use rustical_store::CalendarStore;
use rustical_store::auth::User;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarSetResource {
    pub(crate) principal: String,
    pub(crate) read_only: bool,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Common(CommonPropertiesProp),
}

impl Resource for CalendarSetResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type PrincipalResource = PrincipalResource;
    type Principal = User;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(
            Some(rustical_dav::namespace::NS_DAV),
            "collection",
        )])
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                <Self as CommonPropertiesExtension>::get_prop(self, rmap, user, prop)?,
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

pub struct CalendarSetResourceService<C: CalendarStore> {
    cal_store: Arc<C>,
}

impl<C: CalendarStore> CalendarSetResourceService<C> {
    pub fn new(cal_store: Arc<C>) -> Self {
        Self { cal_store }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore> ResourceService for CalendarSetResourceService<C> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = CalendarSetResource;
    type Error = Error;
    type Principal = User;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        Ok(CalendarSetResource {
            principal: principal.to_owned(),
            read_only: self.cal_store.is_read_only(),
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let calendars = self.cal_store.get_calendars(principal).await?;
        Ok(calendars
            .into_iter()
            .map(|cal| {
                (
                    cal.id.to_owned(),
                    CalendarResource {
                        cal,
                        read_only: self.cal_store.is_read_only(),
                    },
                )
            })
            .collect())
    }
}
