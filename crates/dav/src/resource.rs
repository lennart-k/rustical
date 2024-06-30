use crate::xml::TagList;
use crate::Error;
use actix_web::{http::StatusCode, HttpRequest, ResponseError};
use async_trait::async_trait;
use core::fmt;
use itertools::Itertools;
use rustical_auth::AuthInfo;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::VariantNames;

#[async_trait(?Send)]
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
    type File: Resource<Error = Self::Error>;
    type Error: ResponseError + From<crate::Error> + From<anyhow::Error>;

    async fn new(
        req: &HttpRequest,
        auth_info: &AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error>;

    async fn get_members(
        &self,
        _auth_info: AuthInfo,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(vec![])
    }

    async fn get_file(&self) -> Result<Self::File, Self::Error>;
    async fn save_file(&self, file: Self::File) -> Result<(), Self::Error>;
    async fn delete_file(&self, _use_trashbin: bool) -> Result<(), Self::Error> {
        Err(crate::Error::Unauthorized.into())
    }
}

#[derive(Serialize)]
pub struct PropTagWrapper<T: Serialize> {
    #[serde(rename = "$value")]
    prop: T,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PropWrapper<T: Serialize> {
    Prop(PropTagWrapper<T>),
    TagList(TagList),
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PropstatElement<T: Serialize> {
    pub prop: T,
    pub status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PropstatResponseElement<T1: Serialize, T2: Serialize> {
    pub href: String,
    pub propstat: Vec<PropstatType<T1, T2>>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PropstatType<T1: Serialize, T2: Serialize> {
    Normal(PropstatElement<T1>),
    NotFound(PropstatElement<T2>),
    Conflict(PropstatElement<T2>),
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
        props: Vec<&str>,
    ) -> Result<PropstatResponseElement<PropWrapper<Vec<R::Prop>>, TagList>, R::Error> {
        let mut props = props;
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
            return Ok(PropstatResponseElement {
                href: path,
                propstat: vec![PropstatType::Normal(PropstatElement {
                    prop: PropWrapper::TagList(TagList::from(props)),
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                })],
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

        let mut propstats = vec![PropstatType::Normal(PropstatElement {
            status: format!("HTTP/1.1 {}", StatusCode::OK),
            prop: PropWrapper::Prop(PropTagWrapper {
                prop: prop_responses,
            }),
        })];
        if !invalid_props.is_empty() {
            propstats.push(PropstatType::NotFound(PropstatElement {
                status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                prop: invalid_props
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect_vec()
                    .into(),
            }));
        }
        Ok(PropstatResponseElement {
            href: path,
            propstat: propstats,
        })
    }
}
