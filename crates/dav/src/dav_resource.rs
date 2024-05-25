use crate::{error::Error, xml_snippets::TagList};
use actix_web::{http::StatusCode, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use itertools::Itertools;
use rustical_auth::AuthInfo;
use serde::Serialize;
use std::str::FromStr;
use strum::VariantNames;

#[async_trait(?Send)]
pub trait Resource {
    type PropType: FromStr + VariantNames + Into<&'static str> + Clone;
    type PropResponse: Serialize;

    fn list_dead_props() -> &'static [&'static str] {
        Self::PropType::VARIANTS
    }

    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse>;

    fn get_path(&self) -> &str;
}

// A resource is identified by a URI and has properties
// A resource can also be a collection
// A resource cannot be none, only Methods like PROPFIND, GET, REPORT, etc. can be exposed
// A resource exists
#[async_trait(?Send)]
pub trait ResourceService: Sized {
    type MemberType: Resource;
    type PathComponents: Sized + Clone; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type File: Resource;

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        path_components: Self::PathComponents,
        prefix: String,
    ) -> Result<Self, Error>;

    async fn get_file(&self) -> Result<Self::File>;

    async fn get_members(&self, auth_info: AuthInfo) -> Result<Vec<Self::MemberType>>;
}

#[derive(Serialize)]
pub struct PropWrapper<T: Serialize> {
    #[serde(rename = "$value")]
    prop: T,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PropstatElement<T: Serialize> {
    prop: T,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PropstatResponseElement<T1: Serialize, T2: Serialize> {
    href: String,
    propstat: Vec<PropstatType<T1, T2>>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum PropstatType<T1: Serialize, T2: Serialize> {
    Normal(PropstatElement<T1>),
    NotFound(PropstatElement<T2>),
}

#[async_trait(?Send)]
pub trait HandlePropfind {
    async fn propfind(&self, props: Vec<&str>) -> Result<impl Serialize>;
}

#[async_trait(?Send)]
impl<R: Resource> HandlePropfind for R {
    async fn propfind(
        &self,
        props: Vec<&str>,
    ) -> Result<PropstatResponseElement<PropWrapper<Vec<R::PropResponse>>, TagList>> {
        let mut props = props.into_iter().unique().collect_vec();
        if props.contains(&"allprops") {
            if props.len() != 1 {
                // allprops MUST be the only queried prop per spec
                return Err(anyhow!("allprops MUST be the only queried prop"));
            }
            props = R::list_dead_props().into();
        }

        let mut invalid_props = Vec::<&str>::new();
        let mut prop_responses = Vec::new();
        for prop in props {
            if let Ok(valid_prop) = R::PropType::from_str(prop) {
                match self.get_prop(valid_prop.clone()) {
                    Ok(response) => {
                        prop_responses.push(response);
                    }
                    Err(_) => invalid_props.push(prop),
                }
            } else {
                invalid_props.push(prop);
            }
        }

        let mut propstats = Vec::new();
        propstats.push(PropstatType::Normal(PropstatElement {
            status: format!("HTTP/1.1 {}", StatusCode::OK),
            prop: PropWrapper {
                prop: prop_responses,
            },
        }));
        if !invalid_props.is_empty() {
            propstats.push(PropstatType::NotFound(PropstatElement {
                status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                prop: TagList(invalid_props.iter().map(|&s| s.to_owned()).collect()),
            }));
        }
        Ok(PropstatResponseElement {
            href: self.get_path().to_owned(),
            propstat: propstats,
        })
    }
}
