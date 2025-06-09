use super::{PrincipalUri, Resource};
use crate::Principal;
use crate::resource::{AxumMethods, AxumService};
use async_trait::async_trait;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::response::IntoResponse;
use serde::Deserialize;

#[async_trait]
pub trait ResourceService: Clone + Sized + Send + Sync + AxumMethods + 'static {
    type PathComponents: for<'de> Deserialize<'de> + Sized + Send + Sync + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type MemberType: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Resource: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Error: From<crate::Error> + Send + Sync + IntoResponse + 'static;
    type Principal: Principal + FromRequestParts<Self>;
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
        Self: AxumMethods,
    {
        AxumService::new(self)
    }

    fn axum_router<S: Send + Sync + Clone + 'static>(self) -> Router<S> {
        Router::new().route_service("/", self.axum_service())
    }
}
