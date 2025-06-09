use crate::Principal;
use crate::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use crate::privileges::UserPrivilegeSet;
use crate::resource::{AxumMethods, PrincipalUri, Resource, ResourceName, ResourceService};
use crate::xml::{Resourcetype, ResourcetypeInner};
use async_trait::async_trait;
use axum::Router;
use axum::extract::FromRequestParts;
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

    const IS_COLLECTION: bool = true;

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
pub struct RootResourceService<PRS: ResourceService + Clone, P: Principal, PURI: PrincipalUri>(
    PRS,
    PhantomData<P>,
    PhantomData<PURI>,
);

impl<PRS: ResourceService + Clone, P: Principal, PURI: PrincipalUri>
    RootResourceService<PRS, P, PURI>
{
    pub fn new(principal_resource_service: PRS) -> Self {
        Self(principal_resource_service, PhantomData, PhantomData)
    }
}

#[async_trait]
impl<
    PRS: ResourceService<Principal = P> + Clone,
    P: Principal + FromRequestParts<Self>,
    PURI: PrincipalUri,
> ResourceService for RootResourceService<PRS, P, PURI>
where
    PRS::Resource: ResourceName,
{
    type PathComponents = ();
    type MemberType = PRS::Resource;
    type Resource = RootResource<PRS::Resource, P>;
    type Error = PRS::Error;
    type Principal = P;
    type PrincipalUri = PURI;

    const DAV_HEADER: &str = "1, 3, access-control";

    async fn get_resource(&self, _: &()) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource::<PRS::Resource, P>::default())
    }

    fn axum_router<S: Send + Sync + Clone + 'static>(self) -> Router<S> {
        Router::new()
            .nest("/principal/{principal}", self.0.clone().axum_router())
            .route_service("/", self.axum_service())
    }
}

impl<PRS: ResourceService<Principal = P> + Clone, P: Principal, PURI: PrincipalUri> AxumMethods
    for RootResourceService<PRS, P, PURI>
{
}
