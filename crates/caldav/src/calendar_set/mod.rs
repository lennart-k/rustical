use crate::calendar::resource::CalendarResource;
use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{NamedRoute, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_store::CalendarStore;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

#[derive(Clone)]
pub struct CalendarSetResource {
    pub(crate) principal: String,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(PrincipalPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
pub enum PrincipalProp {
    // WebDAV Access Control (RFC 3744)
    #[strum_discriminants(strum(serialize = "principal-URL"))]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalUrl(HrefElement),
}

impl Resource for CalendarSetResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(
            rustical_dav::namespace::NS_DAV,
            "collection",
        )])
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(
            Self::PrincipalResource::get_url(rmap, vec![&self.principal]).unwrap(),
        );

        Ok(match prop {
            PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

pub struct CalendarSetResourceService<C: CalendarStore + ?Sized> {
    cal_store: Arc<C>,
}

impl<C: CalendarStore + ?Sized> CalendarSetResourceService<C> {
    pub fn new(cal_store: Arc<C>) -> Self {
        Self { cal_store }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarSetResourceService<C> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = CalendarSetResource;
    type Error = Error;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        Ok(CalendarSetResource {
            principal: principal.to_owned(),
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
