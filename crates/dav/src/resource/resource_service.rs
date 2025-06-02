#[cfg(feature = "actix")]
use super::methods::{actix_route_delete, actix_route_propfind, actix_route_proppatch};
use super::{PrincipalUri, Resource};
use crate::Principal;
#[cfg(feature = "actix")]
use actix_web::{http::Method, web, web::Data};
use async_trait::async_trait;
use serde::Deserialize;
use std::str::FromStr;

#[async_trait(?Send)]
pub trait ResourceService: Sized + 'static {
    type MemberType: Resource<Error = Self::Error, Principal = Self::Principal>;
    type PathComponents: for<'de> Deserialize<'de> + Sized + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Error: From<crate::Error>;
    type Principal: Principal;
    type PrincipalUri: PrincipalUri;

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

    #[cfg(feature = "actix")]
    #[inline]
    fn actix_resource(self) -> actix_web::Resource
    where
        Self::Error: actix_web::ResponseError,
        Self::Principal: actix_web::FromRequest,
    {
        web::resource("")
            .app_data(Data::new(self))
            .route(
                web::method(Method::from_str("PROPFIND").unwrap()).to(actix_route_propfind::<Self>),
            )
            .route(
                web::method(Method::from_str("PROPPATCH").unwrap())
                    .to(actix_route_proppatch::<Self>),
            )
            .delete(actix_route_delete::<Self>)
    }

    #[cfg(feature = "actix")]
    fn actix_scope(self) -> actix_web::Scope
    where
        Self::Error: actix_web::ResponseError,
        Self::Principal: actix_web::FromRequest;
}
