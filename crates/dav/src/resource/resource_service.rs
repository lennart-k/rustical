use super::methods::{route_delete, route_propfind, route_proppatch};
use super::{PrincipalUri, Resource};
use crate::Principal;
use actix_web::web::Data;
use actix_web::{ResponseError, http::Method, web};
use async_trait::async_trait;
use serde::Deserialize;
use std::str::FromStr;

#[async_trait(?Send)]
pub trait ResourceService: Sized + 'static {
    type MemberType: Resource<Error = Self::Error, Principal = Self::Principal>;
    type PathComponents: for<'de> Deserialize<'de> + Sized + Clone + 'static; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error, Principal = Self::Principal>;
    type Error: ResponseError + From<crate::Error>;
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

    #[inline]
    fn actix_resource(self) -> actix_web::Resource {
        web::resource("")
            .app_data(Data::new(self))
            .route(web::method(Method::from_str("PROPFIND").unwrap()).to(route_propfind::<Self>))
            .route(web::method(Method::from_str("PROPPATCH").unwrap()).to(route_proppatch::<Self>))
            .delete(route_delete::<Self>)
    }

    fn actix_scope(self) -> actix_web::Scope;
}
