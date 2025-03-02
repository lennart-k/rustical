use std::sync::Arc;

use super::{Resource, ResourceServiceRouter};
use async_trait::async_trait;
use axum::response::IntoResponse;
use rustical_store::auth::AuthenticationProvider;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait ResourceService: Sized + Send + Sync + Clone + 'static {
    type MemberType: Resource<Error = Self::Error>;
    type PathComponents: DeserializeOwned + Sized + Send + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error>;
    type Error: From<crate::Error> + IntoResponse;

    async fn get_members(
        &self,
        _path_components: &Self::PathComponents,
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

    #[inline]
    fn axum_service<AP: AuthenticationProvider>(
        self,
        auth_provider: Arc<AP>,
    ) -> ResourceServiceRouter {
        ResourceServiceRouter::new(self, auth_provider)
    }
}
