use crate::Principal;
use crate::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use crate::privileges::UserPrivilegeSet;
use crate::resource::{PrincipalUri, Resource, ResourceService};
use crate::xml::{Resourcetype, ResourcetypeInner};
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RootResource<PR: Resource, P: Principal>(PhantomData<PR>, PhantomData<P>);

impl<PR: Resource, P: Principal> Default for RootResource<PR, P> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<PR: Resource, P: Principal> Resource for RootResource<PR, P> {
    type Prop = CommonPropertiesProp;
    type Error = PR::Error;
    type Principal = P;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(
            Some(crate::namespace::NS_DAV),
            "collection",
        )])
    }

    fn get_prop(
        &self,
        principal_uri: &impl PrincipalUri,
        user: &P,
        prop: &CommonPropertiesPropName,
    ) -> Result<Self::Prop, Self::Error> {
        CommonPropertiesExtension::get_prop(self, principal_uri, user, prop)
    }

    fn get_user_privileges(&self, _user: &P) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::all())
    }
}

#[derive(Clone)]
pub struct RootResourceService<PR: Resource, P: Principal, PURI: PrincipalUri>(
    PhantomData<PR>,
    PhantomData<P>,
    PhantomData<PURI>,
);

impl<PR: Resource, P: Principal, PURI: PrincipalUri> Default for RootResourceService<PR, P, PURI> {
    fn default() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
}

#[async_trait(?Send)]
impl<PR: Resource<Principal = P>, P: Principal, PURI: PrincipalUri> ResourceService
    for RootResourceService<PR, P, PURI>
{
    type PathComponents = ();
    type MemberType = PR;
    type Resource = RootResource<PR, P>;
    type Error = PR::Error;
    type Principal = P;
    type PrincipalUri = PURI;

    async fn get_resource(&self, _: &()) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource::<PR, P>::default())
    }
}
