use std::marker::PhantomData;

use actix_web::dev::ResourceMap;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

use crate::{
    extension::ResourceExtension,
    privileges::UserPrivilegeSet,
    resource::{InvalidProperty, Resource},
    xml::HrefElement,
};

#[derive(Debug, Clone)]
pub struct CommonPropertiesExtension<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for CommonPropertiesExtension<PR> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CommonPropertiesProp {
    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    CurrentUserPrivilegeSet(UserPrivilegeSet),

    #[serde(untagged)]
    Invalid,
}

impl InvalidProperty for CommonPropertiesProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CommonPropertiesPropName {
    CurrentUserPrincipal,
    CurrentUserPrivilegeSet,
}

impl<R: Resource, PR: Resource> ResourceExtension<R> for CommonPropertiesExtension<PR>
where
    R::PropName: TryInto<CommonPropertiesPropName>,
    R::Prop: From<CommonPropertiesProp>,
{
    type Prop = CommonPropertiesProp;
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
            CommonPropertiesPropName::CurrentUserPrincipal => {
                CommonPropertiesProp::CurrentUserPrincipal(
                    PR::get_url(rmap, &[&user.id]).unwrap().into(),
                )
            }
            CommonPropertiesPropName::CurrentUserPrivilegeSet => {
                CommonPropertiesProp::CurrentUserPrivilegeSet(resource.get_user_privileges(user)?)
            }
        })
    }
}
