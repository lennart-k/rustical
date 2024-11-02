use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum RootPropName {
    Resourcetype,
    CurrentUserPrincipal,
    CurrentUserPrivilegeSet,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RootProp {
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    CurrentUserPrivilegeSet(UserPrivilegeSet),

    #[serde(untagged)]
    Invalid,
}

impl InvalidProperty for RootProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone)]
pub struct RootResource;

impl Resource for RootResource {
    type PropName = RootPropName;
    type Prop = RootProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            RootPropName::Resourcetype => RootProp::Resourcetype(Resourcetype::default()),
            RootPropName::CurrentUserPrincipal => RootProp::CurrentUserPrincipal(
                PrincipalResource::get_principal_url(rmap, &user.id).into(),
            ),
            RootPropName::CurrentUserPrivilegeSet => {
                RootProp::CurrentUserPrivilegeSet(self.get_user_privileges(user)?)
            }
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "carddav_root"
    }

    fn get_user_privileges(&self, _user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::all())
    }
}

pub struct RootResourceService;

#[async_trait(?Send)]
impl ResourceService for RootResourceService {
    type PathComponents = ();
    type MemberType = PrincipalResource;
    type Resource = RootResource;
    type Error = Error;

    async fn new(
        _req: &HttpRequest,
        _path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource)
    }

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
