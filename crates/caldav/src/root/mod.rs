use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use derive_more::derive::{From, TryInto};
use rustical_dav::extension::BoxedExtension;
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

#[derive(EnumString, Debug, VariantNames, Clone, From, TryInto)]
#[strum(serialize_all = "kebab-case")]
pub enum RootPropName {
    Resourcetype,
    #[from]
    #[try_into]
    #[strum(disabled)]
    ExtCommonProperties(CommonPropertiesPropName),
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

#[derive(Deserialize, Serialize, Debug, From, TryInto)]
#[serde(rename_all = "kebab-case")]
pub enum RootProp {
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),

    #[serde(skip_deserializing, untagged)]
    #[from]
    #[try_into]
    ExtCommonProperties(CommonPropertiesProp),

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

    fn list_extensions() -> Vec<BoxedExtension<Self>> {
        vec![BoxedExtension::from_ext(CommonPropertiesExtension::<
            PrincipalResource,
        >::default())]
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            RootPropName::Resourcetype => RootProp::Resourcetype(Resourcetype::default()),
            _ => panic!("we shouldn't end up here"),
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_root"
    }

    fn get_user_privileges(&self, _user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::all())
    }
}

pub struct RootResourceService;

#[async_trait(?Send)]
impl ResourceService for RootResourceService {
    type PathComponents = ();
    type MemberType = RootResource;
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
