use crate::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use crate::privileges::UserPrivilegeSet;
use crate::resource::{NamedRoute, Resource, ResourceService};
use crate::xml::{Resourcetype, ResourcetypeInner};
use async_trait::async_trait;
use rustical_store::auth::User;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RootResource<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for RootResource<PR> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<PR: Resource + NamedRoute + Clone> Resource for RootResource<PR> {
    type Prop = CommonPropertiesProp;
    type Error = PR::Error;
    type PrincipalResource = PR;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(
            Some(crate::namespace::NS_DAV),
            "collection",
        )])
    }

    fn get_prop(
        &self,
        user: &User,
        prop: &CommonPropertiesPropName,
    ) -> Result<Self::Prop, Self::Error> {
        CommonPropertiesExtension::get_prop(self, user, prop)
    }

    fn get_user_privileges(&self, _user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::all())
    }
}

#[derive(Clone)]
pub struct RootResourceService<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for RootResourceService<PR> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[async_trait]
impl<PR: Resource + NamedRoute + Clone> ResourceService for RootResourceService<PR> {
    type PathComponents = ();
    type MemberType = PR;
    type Resource = RootResource<PR>;
    type Error = PR::Error;

    async fn get_resource(&self, _: &()) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource::<PR>::default())
    }
}
