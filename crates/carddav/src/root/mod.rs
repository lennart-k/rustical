use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use derive_more::{From, TryInto};
use rustical_dav::extension::BoxedExtension;
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

#[derive(EnumString, VariantNames, Clone, From, TryInto)]
#[strum(serialize_all = "kebab-case")]
pub enum RootPropName {
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

#[derive(Deserialize, Serialize, From, TryInto)]
#[serde(rename_all = "kebab-case")]
pub enum RootProp {
    #[serde(skip_deserializing, untagged)]
    #[from]
    #[try_into]
    ExtCommonProperties(CommonPropertiesProp<RootResource>),

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
    type ResourceType = Resourcetype;

    fn list_extensions() -> Vec<BoxedExtension<Self>> {
        vec![BoxedExtension::from_ext(CommonPropertiesExtension::<
            PrincipalResource,
        >::default())]
    }

    fn get_prop(
        &self,
        _rmap: &ResourceMap,
        _user: &User,
        _prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        panic!("we shouldn't end up here")
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
}
