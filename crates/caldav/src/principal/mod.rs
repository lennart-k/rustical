use crate::calendar_set::CalendarSetResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{NamedRoute, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

#[derive(Clone)]
pub struct PrincipalResource {
    principal: String,
    home_set: &'static [&'static str],
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct CalendarHomeSet(#[xml(ty = "untagged", flatten)] Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(PrincipalPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // Scheduling Extensions to CalDAV (RFC 6638)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserType(&'static str),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserAddressSet(HrefElement),

    // WebDAV Access Control (RFC 3744)
    #[strum_discriminants(strum(serialize = "principal-URL"))]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalUrl(HrefElement),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarHomeSet(CalendarHomeSet),
}

impl PrincipalResource {
    pub fn get_principal_url(rmap: &ResourceMap, principal: &str) -> String {
        Self::get_url(rmap, vec![principal]).unwrap()
    }
}

impl NamedRoute for PrincipalResource {
    fn route_name() -> &'static str {
        "caldav_principal"
    }
}

impl Resource for PrincipalResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(rustical_dav::namespace::NS_DAV, "collection"),
            ResourcetypeInner(rustical_dav::namespace::NS_DAV, "principal"),
        ])
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_url = Self::get_url(rmap, vec![&self.principal]).unwrap();
        let home_set = CalendarHomeSet(
            self.home_set
                .iter()
                .map(|&home_name| format!("{}/{}", principal_url, home_name).into())
                .collect(),
        );

        Ok(match prop {
            PrincipalPropName::CalendarUserType => PrincipalProp::CalendarUserType("INDIVIDUAL"),
            PrincipalPropName::Displayname => PrincipalProp::Displayname(self.principal.to_owned()),
            PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_url.into()),
            PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(home_set),
            PrincipalPropName::CalendarUserAddressSet => {
                PrincipalProp::CalendarUserAddressSet(principal_url.into())
            }
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

pub struct PrincipalResourceService(pub &'static [&'static str]);

#[async_trait(?Send)]
impl ResourceService for PrincipalResourceService {
    type PathComponents = (String,);
    type MemberType = CalendarSetResource;
    type Resource = PrincipalResource;
    type Error = Error;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        Ok(PrincipalResource {
            principal: principal.to_owned(),
            home_set: self.0,
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .0
            .iter()
            .map(|&set_name| {
                (
                    set_name.to_string(),
                    CalendarSetResource {
                        principal: principal.to_owned(),
                    },
                )
            })
            .collect())
    }
}
