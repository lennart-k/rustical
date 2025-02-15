use crate::{
    privileges::UserPrivilegeSet,
    resource::{NamedRoute, Resource},
    xml::{HrefElement, Resourcetype},
};
use actix_web::dev::ResourceMap;
use rustical_store::auth::User;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumUnitVariants, EnumVariants)]
#[xml(unit_variants_ident = "CommonPropertiesPropName")]
pub enum CommonPropertiesProp {
    // WebDAV (RFC 2518)
    #[xml(skip_deserializing)]
    #[xml(ns = "crate::namespace::NS_DAV")]
    Resourcetype(Resourcetype),

    // WebDAV Current Principal Extension (RFC 5397)
    // #[xml(ns = "crate::namespace::NS_DAV")]
    // CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    #[xml(skip_deserializing)]
    #[xml(ns = "crate::namespace::NS_DAV")]
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    // #[xml(ns = "crate::namespace::NS_DAV")]
    // Owner(Option<HrefElement>),
}

pub trait CommonPropertiesExtension: Resource {
    fn get_prop(
        &self,
        user: &User,
        prop: &CommonPropertiesPropName,
    ) -> Result<CommonPropertiesProp, <Self as Resource>::Error> {
        Ok(match prop {
            CommonPropertiesPropName::Resourcetype => {
                CommonPropertiesProp::Resourcetype(self.get_resourcetype())
            }
            // CommonPropertiesPropName::CurrentUserPrincipal => {
            //     CommonPropertiesProp::CurrentUserPrincipal(
            //         Self::PrincipalResource::get_url(rmap, [&user.id])
            //             .unwrap()
            //             .into(),
            //     )
            // }
            CommonPropertiesPropName::CurrentUserPrivilegeSet => {
                CommonPropertiesProp::CurrentUserPrivilegeSet(self.get_user_privileges(user)?)
            } // CommonPropertiesPropName::Owner => {
              //     CommonPropertiesProp::Owner(self.get_owner().map(|owner| {
              //         Self::PrincipalResource::get_url(rmap, [owner])
              //             .unwrap()
              //             .into()
              //     }))
              // }
        })
    }

    fn set_prop(&self, _prop: CommonPropertiesProp) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&self, _prop: &CommonPropertiesPropName) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }
}

impl<T: Resource> CommonPropertiesExtension for T {}
