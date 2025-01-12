use crate::privileges::UserPrivilegeSet;
use crate::resource::{NamedRoute, Resource, ResourceService};
use crate::xml::{Resourcetype, ResourcetypeInner};
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_store::auth::User;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use serde::Serialize;
use std::marker::PhantomData;
use strum::{EnumString, IntoStaticStr, VariantNames};

#[derive(Clone)]
pub struct RootResource<PR: Resource>(PhantomData<PR>);

impl<PR: Resource> Default for RootResource<PR> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(EnumString, VariantNames, Clone, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum RootResourcePropName {}

#[derive(XmlDeserialize, XmlSerialize, Serialize, Clone, PartialEq)]
pub enum RootResourceProp {}

impl From<RootResourceProp> for RootResourcePropName {
    fn from(_value: RootResourceProp) -> Self {
        unreachable!()
    }
}

impl<PR: Resource + NamedRoute> Resource for RootResource<PR> {
    type PropName = RootResourcePropName;
    type Prop = RootResourceProp;
    type Error = PR::Error;
    type PrincipalResource = PR;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[ResourcetypeInner(crate::namespace::NS_DAV, "collection")])
    }

    fn get_prop(
        &self,
        _rmap: &ResourceMap,
        _user: &User,
        _prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        unreachable!("we shouldn't end up here")
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

#[async_trait(?Send)]
impl<PR: Resource + NamedRoute> ResourceService for RootResourceService<PR> {
    type PathComponents = ();
    type MemberType = PR;
    type Resource = RootResource<PR>;
    type Error = PR::Error;

    async fn get_resource(&self, _: &()) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource::<PR>::default())
    }
}
