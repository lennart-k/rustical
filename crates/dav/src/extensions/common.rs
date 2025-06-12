use crate::{
    Principal,
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource},
    xml::{HrefElement, Resourcetype},
};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, PropName, EnumVariants)]
#[xml(unit_variants_ident = "CommonPropertiesPropName")]
pub enum CommonPropertiesProp {
    // WebDAV (RFC 2518)
    #[xml(skip_deserializing)]
    #[xml(ns = "crate::namespace::NS_DAV")]
    Resourcetype(Resourcetype),
    #[xml(ns = "crate::namespace::NS_DAV")]
    Displayname(Option<String>),

    // WebDAV Current Principal Extension (RFC 5397)
    #[xml(ns = "crate::namespace::NS_DAV")]
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    #[xml(skip_deserializing)]
    #[xml(ns = "crate::namespace::NS_DAV")]
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    #[xml(ns = "crate::namespace::NS_DAV")]
    Owner(Option<HrefElement>),
}

pub trait CommonPropertiesExtension: Resource {
    fn get_prop(
        &self,
        principal_uri: &impl PrincipalUri,
        principal: &Self::Principal,
        prop: &CommonPropertiesPropName,
    ) -> Result<CommonPropertiesProp, <Self as Resource>::Error> {
        Ok(match prop {
            CommonPropertiesPropName::Resourcetype => {
                CommonPropertiesProp::Resourcetype(self.get_resourcetype())
            }
            CommonPropertiesPropName::Displayname => {
                CommonPropertiesProp::Displayname(self.get_displayname().map(|s| s.to_string()))
            }
            CommonPropertiesPropName::CurrentUserPrincipal => {
                CommonPropertiesProp::CurrentUserPrincipal(
                    principal_uri.principal_uri(principal.get_id()).into(),
                )
            }
            CommonPropertiesPropName::CurrentUserPrivilegeSet => {
                CommonPropertiesProp::CurrentUserPrivilegeSet(self.get_user_privileges(principal)?)
            }
            CommonPropertiesPropName::Owner => CommonPropertiesProp::Owner(
                self.get_owner()
                    .map(|owner| principal_uri.principal_uri(owner).into()),
            ),
        })
    }

    fn set_prop(&mut self, prop: CommonPropertiesProp) -> Result<(), crate::Error> {
        match prop {
            CommonPropertiesProp::Displayname(name) => self.set_displayname(name),
            _ => Err(crate::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: &CommonPropertiesPropName) -> Result<(), crate::Error> {
        match prop {
            CommonPropertiesPropName::Displayname => self.set_displayname(None),
            _ => Err(crate::Error::PropReadOnly),
        }
    }
}

impl<R: Resource> CommonPropertiesExtension for R {}
