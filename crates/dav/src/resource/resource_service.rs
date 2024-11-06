use std::str::FromStr;

use actix_web::{dev::ResourceMap, http::Method, web, HttpRequest, ResponseError};
use async_trait::async_trait;
use serde::Deserialize;

use crate::methods::{route_delete, route_propfind, route_proppatch};

use super::Resource;

#[async_trait(?Send)]
pub trait ResourceService: Sized + 'static {
    type MemberType: Resource<Error = Self::Error>;
    type PathComponents: for<'de> Deserialize<'de> + Sized + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error>;
    type Error: ResponseError + From<crate::Error>;

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error>;

    async fn get_members(
        &self,
        _rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![])
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error>;
    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }
    async fn delete_resource(&self, _use_trashbin: bool) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }

    #[inline]
    fn resource_name() -> &'static str {
        Self::Resource::resource_name()
    }

    #[inline]
    fn actix_resource() -> actix_web::Resource {
        Self::actix_additional_routes(
            web::resource("")
                .name(Self::resource_name())
                .route(
                    web::method(Method::from_str("PROPFIND").unwrap()).to(route_propfind::<Self>),
                )
                .route(
                    web::method(Method::from_str("PROPPATCH").unwrap()).to(route_proppatch::<Self>),
                )
                .delete(route_delete::<Self>),
        )
    }

    /// Hook for other resources to insert their additional methods (i.e. REPORT, MKCALENDAR)
    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res
    }
}
