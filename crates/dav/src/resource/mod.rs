use crate::extension::ResourceExtension;
use crate::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use crate::methods::{route_delete, route_propfind, route_proppatch};
use crate::privileges::UserPrivilegeSet;
use crate::xml::multistatus::{PropTagWrapper, PropstatElement, PropstatWrapper};
use crate::xml::{multistatus::ResponseElement, TagList};
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::error::UrlGenerationError;
use actix_web::http::Method;
use actix_web::test::TestRequest;
use actix_web::web;
use actix_web::{http::StatusCode, HttpRequest, ResponseError};
use async_trait::async_trait;
use itertools::Itertools;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::VariantNames;

pub trait ResourceReadProp: Serialize + InvalidProperty {}
impl<T: Serialize + InvalidProperty> ResourceReadProp for T {}

pub trait ResourceProp: ResourceReadProp + for<'de> Deserialize<'de> {}
impl<T: ResourceReadProp + for<'de> Deserialize<'de>> ResourceProp for T {}

pub trait ResourcePropName: FromStr + VariantNames {}
impl<T: FromStr + VariantNames> ResourcePropName for T {}

pub trait ResourceType: Serialize + for<'de> Deserialize<'de> {}
impl<T: Serialize + for<'de> Deserialize<'de>> ResourceType for T {}

pub trait Resource: Clone + 'static {
    type PropName: ResourcePropName;
    type Prop: ResourceProp + From<CommonPropertiesProp<Self::ResourceType>>;
    type Error: ResponseError + From<crate::Error>;
    type PrincipalResource: Resource;
    type ResourceType: Default + Serialize + for<'de> Deserialize<'de>;

    fn list_extensions() -> Vec<impl ResourceExtension<Self>> {
        vec![CommonPropertiesExtension::default()]
    }

    fn list_props() -> Vec<&'static str> {
        Self::PropName::VARIANTS.iter().map(|&prop| prop).collect()
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error>;

    fn set_prop(&mut self, _prop: Self::Prop) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&mut self, _prop: &Self::PropName) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn resource_name() -> &'static str;

    fn get_owner(&self) -> Option<&str> {
        None
    }

    fn get_url<U, I>(rmap: &ResourceMap, elements: U) -> Result<String, UrlGenerationError>
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        Ok(rmap
            .url_for(
                &TestRequest::default().to_http_request(),
                Self::resource_name(),
                elements,
            )?
            .path()
            .to_owned())
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error>;

    fn propfind(
        &self,
        path: &str,
        mut props: Vec<&str>,
        user: &User,
        rmap: &ResourceMap,
    ) -> Result<ResponseElement<PropstatWrapper<Self::Prop>>, Self::Error> {
        if props.contains(&"propname") {
            if props.len() != 1 {
                // propname MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("propname MUST be the only queried prop".to_owned()).into(),
                );
            }
            let mut props: Vec<String> = Self::list_props()
                .iter()
                .map(|&prop| prop.to_string())
                .collect();
            for extension in Self::list_extensions() {
                let ext_props: Vec<String> = extension
                    .list_props()
                    .iter()
                    .map(|&prop| prop.to_string())
                    .collect();
                props.extend(ext_props);
            }
            return Ok(ResponseElement {
                href: path.to_owned(),
                propstat: vec![PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props),
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                })],
                ..Default::default()
            });
        }
        if props.contains(&"allprop") {
            if props.len() != 1 {
                // allprop MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("allprop MUST be the only queried prop".to_owned()).into(),
                );
            }
            props = Self::list_props().into();
            for extension in Self::list_extensions() {
                let ext_props: Vec<&str> = extension.list_props().into();
                props.extend(ext_props);
            }
        }

        let mut prop_responses = Vec::new();

        for extension in Self::list_extensions() {
            let (ext_invalid_props, ext_responses) = extension.propfind(self, props, user, rmap)?;
            props = ext_invalid_props;
            prop_responses.extend(ext_responses);
        }

        let (valid_props, invalid_props): (Vec<Option<Self::PropName>>, Vec<Option<&str>>) = props
            .into_iter()
            .map(|prop| {
                if let Ok(valid_prop) = Self::PropName::from_str(prop) {
                    (Some(valid_prop), None)
                } else {
                    (None, Some(prop))
                }
            })
            .unzip();
        let valid_props: Vec<Self::PropName> = valid_props.into_iter().flatten().collect();
        let invalid_props: Vec<&str> = invalid_props.into_iter().flatten().collect();

        prop_responses.extend(
            valid_props
                .into_iter()
                .map(|prop| self.get_prop(rmap, user, &prop))
                .collect::<Result<Vec<Self::Prop>, Self::Error>>()?,
        );

        let mut propstats = vec![PropstatWrapper::Normal(PropstatElement {
            status: format!("HTTP/1.1 {}", StatusCode::OK),
            prop: PropTagWrapper {
                prop: prop_responses,
            },
        })];
        if !invalid_props.is_empty() {
            propstats.push(PropstatWrapper::TagList(PropstatElement {
                status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                prop: invalid_props
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect_vec()
                    .into(),
            }));
        }
        Ok(ResponseElement {
            href: path.to_owned(),
            propstat: propstats,
            ..Default::default()
        })
    }
}

pub trait InvalidProperty {
    fn invalid_property(&self) -> bool;
}

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
