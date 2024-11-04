use std::any::type_name;
use std::marker::PhantomData;

use crate::extension::BoxedExtension;
use crate::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use crate::privileges::UserPrivilegeSet;
use crate::resource::{Resource, ResourceService};
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

#[derive(Clone)]
pub struct RootResource<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for RootResource<PR> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<PR: Resource> Resource for RootResource<PR> {
    type PropName = CommonPropertiesPropName;
    type Prop = CommonPropertiesProp<Self::ResourceType>;
    type Error = PR::Error;
    type ResourceType = Resourcetype;
    type PrincipalResource = PR;

    fn list_extensions() -> Vec<BoxedExtension<Self>> {
        vec![BoxedExtension::from_ext(
            CommonPropertiesExtension::<Self>::default(),
        )]
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
        type_name::<Self>()
    }

    fn get_user_privileges(&self, _user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::all())
    }
}

#[derive(Default)]
pub struct RootResourceService<PR: Resource>(PhantomData<PR>);

#[async_trait(?Send)]
impl<PR: Resource> ResourceService for RootResourceService<PR> {
    type PathComponents = ();
    type MemberType = PR;
    type Resource = RootResource<PR>;
    type Error = PR::Error;

    async fn new(
        _req: &HttpRequest,
        _path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        Ok(Self(Default::default()))
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource::<PR>::default())
    }
}
