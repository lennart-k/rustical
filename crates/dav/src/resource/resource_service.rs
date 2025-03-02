use std::sync::Arc;

use super::{Resource, ResourceServiceRouter};
use actix_web::error::UrlGenerationError;
use actix_web::test::TestRequest;
use actix_web::{dev::ResourceMap, ResponseError};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use rustical_store::auth::AuthenticationProvider;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait ResourceService: Sized + Send + Sync + Clone + 'static {
    type MemberType: Resource<Error = Self::Error>;
    type PathComponents: DeserializeOwned + Sized + Send + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error>;
    type Error: ResponseError + From<crate::Error> + IntoResponse;

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

    /// Hook for other resources to insert their additional methods (i.e. REPORT, MKCALENDAR)
    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res
    }

    #[inline]
    fn axum_service<AP: AuthenticationProvider>(
        self,
        auth_provider: Arc<AP>,
    ) -> ResourceServiceRouter {
        ResourceServiceRouter::new(self, auth_provider)
    }
}

pub trait NamedRoute {
    fn route_name() -> &'static str;

    fn get_url<U, I>(rmap: &ResourceMap, elements: U) -> Result<String, UrlGenerationError>
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        Ok(rmap
            .url_for(
                &TestRequest::default().to_http_request(),
                Self::route_name(),
                elements,
            )?
            .path()
            .to_owned())
    }
}
