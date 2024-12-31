use crate::calendar::resource::CalendarResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::auth::User;
use rustical_store::CalendarStore;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

pub struct PrincipalResourceService<C: CalendarStore + ?Sized> {
    principal: String,
    cal_store: Arc<C>,
}

#[derive(Clone)]
pub struct PrincipalResource {
    principal: String,
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

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarHomeSet(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserAddressSet(HrefElement),
}

impl PrincipalResource {
    pub fn get_principal_url(rmap: &ResourceMap, principal: &str) -> String {
        Self::get_url(rmap, vec![principal]).unwrap()
    }
}

impl Resource for PrincipalResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> &'static [&'static str] {
        &["collection", "principal"]
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(Self::get_url(rmap, vec![&self.principal]).unwrap());

        Ok(match prop {
            PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
            PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(principal_href),
            PrincipalPropName::CalendarUserAddressSet => {
                PrincipalProp::CalendarUserAddressSet(principal_href)
            }
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_principal"
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for PrincipalResourceService<C> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = PrincipalResource;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        (principal,): Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let cal_store = req
            .app_data::<Data<C>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            principal,
        })
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        Ok(PrincipalResource {
            principal: self.principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let calendars = self.cal_store.get_calendars(&self.principal).await?;
        Ok(calendars
            .into_iter()
            .map(|cal| {
                (
                    CalendarResource::get_url(rmap, vec![&self.principal, &cal.id]).unwrap(),
                    cal.into(),
                )
            })
            .collect())
    }
}
