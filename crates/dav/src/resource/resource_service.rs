use super::{PrincipalUri, Resource};
use crate::Principal;
use crate::resource::{AxumMethods, AxumService};
use async_trait::async_trait;
use serde::Deserialize;

#[async_trait]
pub trait ResourceService: Sized + Send + Sync + 'static {
    type PathComponents: for<'de> Deserialize<'de> + Sized + Send + Sync + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type MemberType: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Resource: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Error: From<crate::Error> + Send;
    type Principal: Principal;
    type PrincipalUri: PrincipalUri;

    const DAV_HEADER: &'static str;

    async fn get_members(
        &self,
        _path: &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![])
    }

    async fn get_resource(
        &self,
        _path: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error>;

    async fn save_resource(
        &self,
        _path: &Self::PathComponents,
        _file: Self::Resource,
    ) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }

    async fn delete_resource(
        &self,
        _path: &Self::PathComponents,
        _use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }

    fn axum_service(self) -> AxumService<Self>
    where
        Self: Clone + Send + Sync + AxumMethods,
    {
        AxumService::new(self)
    }
}
