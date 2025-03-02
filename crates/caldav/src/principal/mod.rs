use crate::calendar_set::CalendarSetResource;
use crate::Error;
use async_trait::async_trait;
use educe::Educe;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::user::PrincipalType;
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct PrincipalResource {
    principal: User,
    home_set: &'static [(&'static str, bool)],
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct CalendarHomeSet(#[xml(ty = "untagged", flatten)] Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // Scheduling Extensions to CalDAV (RFC 6638)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    CalendarUserType(PrincipalType),
    // #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    // CalendarUserAddressSet(HrefElement),

    // WebDAV Access Control (RFC 3744)
    // #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = b"principal-URL")]
    // PrincipalUrl(HrefElement),

    // CalDAV (RFC 4791)
    // #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    // CalendarHomeSet(CalendarHomeSet),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "principal"),
        ])
    }

    fn get_prop(
        &self,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        // TODO: Reimplement
        // let principal_url = Self::get_url(rmap, vec![&self.principal.id]).unwrap();

        // let home_set = CalendarHomeSet(
        //     user.memberships()
        //         .into_iter()
        //         .map(|principal| Self::get_url(rmap, vec![principal]).unwrap())
        //         .flat_map(|principal_url| {
        //             self.home_set.iter().map(move |&(home_name, _read_only)| {
        //                 HrefElement::new(format!("{}/{}", &principal_url, home_name))
        //             })
        //         })
        //         .collect(),
        // );

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
                    // PrincipalPropName::PrincipalUrl => {
                    //     PrincipalProp::PrincipalUrl(principal_url.into())
                    // }
                    // PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(home_set),
                    // PrincipalPropName::CalendarUserAddressSet => {
                    //     PrincipalProp::CalendarUserAddressSet(principal_url.into())
                    // }
                })
            }
            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                <Self as CommonPropertiesExtension>::get_prop(self, user, prop)?,
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

#[derive(Educe)]
#[educe(Clone)]
pub struct PrincipalResourceService<AP: AuthenticationProvider> {
    pub auth_provider: Arc<AP>,
    pub home_set: &'static [(&'static str, bool)],
}

#[async_trait]
impl<AP: AuthenticationProvider> ResourceService for PrincipalResourceService<AP> {
    type PathComponents = (String,);
    type MemberType = CalendarSetResource;
    type Resource = PrincipalResource;
    type Error = Error;

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
            home_set: self.home_set,
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .home_set
            .iter()
            .map(|&(set_name, read_only)| {
                (
                    set_name.to_string(),
                    CalendarSetResource {
                        principal: principal.to_owned(),
                        read_only,
                    },
                )
            })
            .collect())
    }
}
