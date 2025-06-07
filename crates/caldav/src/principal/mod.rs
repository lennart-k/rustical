use crate::calendar_set::{CalendarSetResource, CalendarSetResourceService};
use crate::{CalDavPrincipalUri, Error};
use actix_web::web;
use async_trait::async_trait;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::user::PrincipalType;
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_store::{CalendarStore, SubscriptionStore};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct PrincipalResource {
    principal: User,
    home_set: &'static [&'static str],
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct CalendarHomeSet(#[xml(ty = "untagged", flatten)] Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // Scheduling Extensions to CalDAV (RFC 6638)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    CalendarUserType(PrincipalType),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserAddressSet(HrefElement),

    // WebDAV Access Control (RFC 3744)
    #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = b"principal-URL")]
    PrincipalUrl(HrefElement),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarHomeSet(CalendarHomeSet),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type Principal = User;

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
                .map(|principal| puri.principal_uri(principal))
                .flat_map(|principal_url| {
                    self.home_set.iter().map(move |&home_name| {
                        HrefElement::new(format!("{}/{}", &principal_url, home_name))
                    })
                })
                .collect(),
        );

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::CalendarUserType => {
                        PrincipalProp::CalendarUserType(self.principal.principal_type.to_owned())
                    }
                    PrincipalPropName::Displayname => PrincipalProp::Displayname(
                        self.principal
                            .displayname
                            .to_owned()
                            .unwrap_or(self.principal.id.to_owned()),
                    ),
                    PrincipalPropName::PrincipalUrl => {
                        PrincipalProp::PrincipalUrl(principal_url.into())
                    }
                    PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(home_set),
                    PrincipalPropName::CalendarUserAddressSet => {
                        PrincipalProp::CalendarUserAddressSet(principal_url.into())
                    }
                })
            }
            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                <Self as CommonPropertiesExtension>::get_prop(self, puri, user, prop)?,
            ),
        })
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

#[derive(Debug)]
pub struct PrincipalResourceService<
    AP: AuthenticationProvider,
    S: SubscriptionStore,
    CS: CalendarStore,
    BS: CalendarStore,
> {
    pub(crate) auth_provider: Arc<AP>,
    pub(crate) sub_store: Arc<S>,
    pub(crate) cal_store: Arc<CS>,
    pub(crate) birthday_store: Arc<BS>,
}

impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore, BS: CalendarStore> Clone
    for PrincipalResourceService<AP, S, CS, BS>
{
    fn clone(&self) -> Self {
        Self {
            auth_provider: self.auth_provider.clone(),
            sub_store: self.sub_store.clone(),
            cal_store: self.cal_store.clone(),
            birthday_store: self.birthday_store.clone(),
        }
    }
}

#[async_trait]
impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore, BS: CalendarStore>
    ResourceService for PrincipalResourceService<AP, S, CS, BS>
{
    type PathComponents = (String,);
    type MemberType = CalendarSetResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control";

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let user = self
            .auth_provider
            .get_principal(principal)
            .await?
            .ok_or(crate::Error::NotFound)?;
        Ok(PrincipalResource {
            principal: user,
            home_set: &["calendar", "birthdays"],
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![
            (
                "calendar".to_owned(),
                CalendarSetResource {
                    principal: principal.to_owned(),
                    read_only: false,
                },
            ),
            (
                "birthdays".to_owned(),
                CalendarSetResource {
                    principal: principal.to_owned(),
                    read_only: true,
                },
            ),
        ])
    }

    fn actix_scope(self) -> actix_web::Scope {
        web::scope("/principal/{principal}")
            .service(
                CalendarSetResourceService::<_, S>::new(
                    "calendar",
                    self.cal_store.clone(),
                    self.sub_store.clone(),
                )
                .actix_scope(),
            )
            .service(
                CalendarSetResourceService::<_, S>::new(
                    "birthdays",
                    self.birthday_store.clone(),
                    self.sub_store.clone(),
                )
                .actix_scope(),
            )
            .service(self.actix_resource())
    }
}
