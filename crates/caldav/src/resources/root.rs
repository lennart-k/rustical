use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::events::BytesText;
use rustical_auth::AuthInfo;
use rustical_dav::{resource::Resource, xml_snippets::write_resourcetype};

pub struct RootResource {
    prefix: String,
    principal: String,
    path: String,
}

#[async_trait(?Send)]
impl Resource for RootResource {
    type UriComponents = ();
    type MemberType = Self;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        _uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self> {
        Ok(Self {
            prefix,
            principal: auth_info.user_id,
            path: req.path().to_string(),
        })
    }

    fn write_prop<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
        prop: &str,
    ) -> Result<()> {
        match prop {
            "resourcetype" => write_resourcetype(writer, vec!["collection"])?,
            "current-user-principal" => {
                writer
                    .create_element("current-user-principal")
                    .write_inner_content(|writer| {
                        writer
                            .create_element("href")
                            .write_text_content(BytesText::new(&format!(
                                "{}/{}",
                                self.prefix, self.principal
                            )))?;
                        Ok(())
                    })?;
            }
            _ => return Err(anyhow!("invalid prop!")),
        };
        Ok(())
    }

    fn list_dead_props() -> Vec<&'static str> {
        vec!["resourcetype", "current-user-principal"]
    }
}
