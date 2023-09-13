use crate::resource::Resource;
use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;

pub struct EventResource {
    path: String,
}

#[async_trait(?Send)]
impl Resource for EventResource {
    type UriComponents = (String, String, String); // principal, calendar, event
    type MemberType = Self;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn acquire_from_request(
        req: HttpRequest,
        _auth_info: AuthInfo,
        _uri_components: Self::UriComponents,
        _prefix: String,
    ) -> Result<Self> {
        Ok(Self {
            path: req.path().to_string(),
        })
    }

    fn write_prop<W: std::io::Write>(
        &self,
        _writer: &mut quick_xml::Writer<W>,
        _prop: &str,
    ) -> Result<()> {
        Err(anyhow!("invalid prop!"))
    }

    fn list_dead_props() -> Vec<&'static str> {
        vec![]
    }
}
