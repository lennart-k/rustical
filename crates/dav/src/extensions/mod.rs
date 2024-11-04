use crate::{
    extension::ResourceExtension,
    privileges::UserPrivilegeSet,
    resource::{InvalidProperty, Resource, ResourceType},
    xml::HrefElement,
};
use actix_web::dev::ResourceMap;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use strum::{EnumString, VariantNames};

#[derive(Clone)]
pub struct CommonPropertiesExtension<R: Resource>(PhantomData<R>);

impl<R: Resource> Default for CommonPropertiesExtension<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CommonPropertiesProp<RT: ResourceType> {
    // WebDAV (RFC 2518)
    #[serde(skip_deserializing)]
    Resourcetype(RT),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    Owner(Option<HrefElement>),

    #[serde(untagged)]
    Invalid,
}

impl<RT: ResourceType> InvalidProperty for CommonPropertiesProp<RT> {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(EnumString, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CommonPropertiesPropName {
    Resourcetype,
    CurrentUserPrincipal,
    CurrentUserPrivilegeSet,
    Owner,
}

impl<R: Resource> ResourceExtension<R> for CommonPropertiesExtension<R>
where
    R::Prop: From<CommonPropertiesProp<R::ResourceType>>,
{
    type Prop = CommonPropertiesProp<R::ResourceType>;
    type PropName = CommonPropertiesPropName;
    type Error = R::Error;

    fn get_prop(
        &self,
        resource: &R,
        rmap: &ResourceMap,
        user: &User,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CommonPropertiesPropName::Resourcetype => {
                CommonPropertiesProp::Resourcetype(R::ResourceType::default())
            }
            CommonPropertiesPropName::CurrentUserPrincipal => {
                CommonPropertiesProp::CurrentUserPrincipal(
                    R::PrincipalResource::get_url(rmap, &[&user.id])
                        .unwrap()
                        .into(),
                )
            }
            CommonPropertiesPropName::CurrentUserPrivilegeSet => {
                CommonPropertiesProp::CurrentUserPrivilegeSet(resource.get_user_privileges(user)?)
            }
            CommonPropertiesPropName::Owner => {
                CommonPropertiesProp::Owner(resource.get_owner().map(|owner| {
                    R::PrincipalResource::get_url(rmap, &[owner])
                        .unwrap()
                        .into()
                }))
            }
        })
    }
}
