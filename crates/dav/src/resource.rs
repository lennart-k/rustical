use actix_web::{http::StatusCode, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::Writer;
use rustical_auth::AuthInfo;
use serde::Serialize;
use std::str::FromStr;
use strum::{EnumProperty, VariantNames};

use crate::xml_snippets::TagList;

// A resource is identified by a URI and has properties
// A resource can also be a collection
// A resource cannot be none, only Methods like PROPFIND, GET, REPORT, etc. can be exposed
// A resource exists
#[async_trait(?Send)]
pub trait Resource: Sized {
    type MemberType: Resource;
    type UriComponents: Sized; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)
    type PropType: FromStr + VariantNames + Into<&'static str> + EnumProperty + Clone;
    type PropResponse: Serialize;

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self>;

    fn get_path(&self) -> &str;
    async fn get_members(&self) -> Result<Vec<Self::MemberType>>;

    fn list_dead_props() -> &'static [&'static str] {
        Self::PropType::VARIANTS
    }
    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse>;
}

#[derive(Serialize)]
struct PropWrapper<T: Serialize> {
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
struct PropstatResponseElement<T: Serialize> {
    href: String,
    propstat: PropstatElement<T>,
}

pub trait HandlePropfind {
    fn propfind(&self, props: Vec<&str>) -> Result<String>;
}

impl<R: Resource> HandlePropfind for R {
    fn propfind(&self, props: Vec<&str>) -> Result<String> {
        let mut props = props;
        if props.contains(&"allprops") {
            if props.len() != 1 {
                // allprops MUST be the only queried prop per spec
                return Err(anyhow!("allprops MUST be the only queried prop"));
            }
            props = R::list_dead_props().into();
        }

        let mut invalid_props = Vec::<&str>::new();

        let mut output_buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);

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

        writer.write_serializable(
            "response",
            &PropstatResponseElement {
                href: self.get_path().to_owned(),
                propstat: PropstatElement {
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                    prop: PropWrapper {
                        prop: prop_responses,
                    },
                },
            },
        )?;
        if !invalid_props.is_empty() {
            // TODO: proper error reporting
            writer.write_serializable(
                "response",
                &PropstatResponseElement {
                    href: self.get_path().to_owned(),
                    propstat: PropstatElement {
                        status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                        prop: TagList(invalid_props.iter().map(|&s| s.to_owned()).collect()),
                    },
                },
            )?;
        }
        Ok(std::str::from_utf8(&output_buffer)?.to_string())
    }
}
