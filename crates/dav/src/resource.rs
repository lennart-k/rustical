use crate::xml::multistatus::{PropTagWrapper, PropstatElement, PropstatWrapper};
use crate::xml::{multistatus::ResponseElement, TagList};
use crate::Error;
use actix_web::{http::StatusCode, HttpRequest, ResponseError};
use async_trait::async_trait;
use core::fmt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::VariantNames;

pub trait Resource: Clone {
    type PropName: FromStr + VariantNames + Clone;
    type Prop: Serialize + for<'de> Deserialize<'de> + fmt::Debug + InvalidProperty;
    type Error: ResponseError + From<crate::Error> + From<anyhow::Error>;

    fn list_props() -> &'static [&'static str] {
        Self::PropName::VARIANTS
    }

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error>;

    fn set_prop(&mut self, _prop: Self::Prop) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&mut self, _prop: Self::PropName) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }
}

pub trait InvalidProperty {
    fn invalid_property(&self) -> bool;
}

// A resource is identified by a URI and has properties
// A resource can also be a collection
// A resource cannot be none, only Methods like PROPFIND, GET, REPORT, etc. can be exposed
// A resource exists
#[async_trait(?Send)]
pub trait ResourceService: Sized {
    type MemberType: Resource<Error = Self::Error>;
    type PathComponents: Sized + Clone; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type Resource: Resource<Error = Self::Error>;
    type Error: ResponseError + From<crate::Error> + From<anyhow::Error>;

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error>;

    async fn get_members(&self) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![])
    }

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Self::Error>;
    async fn save_resource(&self, file: Self::Resource) -> Result<(), Self::Error>;
    async fn delete_resource(&self, _use_trashbin: bool) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }
}

#[async_trait(?Send)]
pub trait HandlePropfind {
    type Error: ResponseError + From<crate::Error> + From<anyhow::Error>;

    async fn propfind(
        &self,
        prefix: &str,
        path: String,
        props: Vec<&str>,
    ) -> Result<impl Serialize, Self::Error>;
}

#[async_trait(?Send)]
impl<R: Resource> HandlePropfind for R {
    type Error = R::Error;

    async fn propfind(
        &self,
        prefix: &str,
        path: String,
        mut props: Vec<&str>,
    ) -> Result<ResponseElement<PropstatWrapper<R::Prop>>, R::Error> {
        if props.contains(&"propname") {
            if props.len() != 1 {
                // propname MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("propname MUST be the only queried prop".to_owned()).into(),
                );
            }
            let props: Vec<String> = R::list_props()
                .iter()
                .map(|&prop| prop.to_string())
                .collect();
            return Ok(ResponseElement {
                href: path,
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
            props = R::list_props().into();
        }

        let (valid_props, invalid_props): (Vec<Option<R::PropName>>, Vec<Option<&str>>) = props
            .into_iter()
            .map(|prop| {
                if let Ok(valid_prop) = R::PropName::from_str(prop) {
                    (Some(valid_prop), None)
                } else {
                    (None, Some(prop))
                }
            })
            .unzip();
        let valid_props: Vec<R::PropName> = valid_props.into_iter().flatten().collect();
        let invalid_props: Vec<&str> = invalid_props.into_iter().flatten().collect();

        let prop_responses = valid_props
            .into_iter()
            .map(|prop| self.get_prop(prefix, prop))
            .collect::<Result<Vec<R::Prop>, R::Error>>()?;

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
            href: path,
            propstat: propstats,
            ..Default::default()
        })
    }
}
