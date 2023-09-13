use std::io::Write;

use actix_web::{http::StatusCode, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::Writer;
use rustical_auth::AuthInfo;

use crate::propfind::{write_invalid_props_response, write_propstat_response};

// A resource is identified by a URI and has properties
// A resource can also be a collection
// A resource cannot be none, only Methods like PROPFIND, GET, REPORT, etc. can be exposed
// A resource exists
#[async_trait(?Send)]
pub trait Resource: Sized {
    type MemberType: Resource;
    type UriComponents: Sized; // defines how the resource URI maps to parameters, i.e. /{principal}/{calendar} -> (String, String)

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self>;

    fn get_path(&self) -> &str;
    async fn get_members(&self) -> Result<Vec<Self::MemberType>>;

    fn list_dead_props() -> Vec<&'static str>;
    fn write_prop<W: Write>(&self, writer: &mut Writer<W>, prop: &str) -> Result<()>;
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
                return Err(anyhow!("allprops MUST be the only quereid prop"));
            }
            props = R::list_dead_props();
        }

        let mut invalid_props = Vec::<&str>::new();

        let mut output_buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut output_buffer, b' ', 2);

        write_propstat_response(&mut writer, self.get_path(), StatusCode::OK, |writer| {
            for prop in props {
                // TODO: Fix error types
                match self
                    .write_prop(writer, prop)
                    .map_err(|_e| quick_xml::Error::TextNotFound)
                {
                    Ok(_) => {}
                    Err(_) => invalid_props.push(prop),
                };
            }
            Ok(())
        })?;
        write_invalid_props_response(&mut writer, self.get_path(), invalid_props)?;
        Ok(std::str::from_utf8(&output_buffer)?.to_string())
    }
}
