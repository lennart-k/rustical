use crate::extensions::CommonPropertiesProp;
use crate::privileges::UserPrivilegeSet;
use crate::resource::{Resource, ResourceService};
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_store::auth::User;
use std::any::type_name;
use std::marker::PhantomData;
use strum::{EnumString, VariantNames};

#[derive(Clone)]
pub struct RootResource<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for RootResource<PR> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(EnumString, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum RootResourcePropName {}

impl<PR: Resource> Resource for RootResource<PR> {
    type PropName = RootResourcePropName;
    type Prop = CommonPropertiesProp;
    type Error = PR::Error;
    type PrincipalResource = PR;

    fn get_resourcetype() -> &'static [&'static str] {
        &["collection"]
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
