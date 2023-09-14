use std::sync::Arc;

use crate::{proptypes::write_href_prop, resource::Resource, xml_snippets::write_resourcetype};
use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::events::BytesText;
use rustical_auth::AuthInfo;
use rustical_store::calendar::CalendarStore;
use tokio::sync::RwLock;

use super::calendar::CalendarResource;

pub struct PrincipalCalendarsResource<C: CalendarStore> {
    prefix: String,
    principal: String,
    path: String,
    cal_store: Arc<RwLock<C>>,
}

#[async_trait(?Send)]
impl<C: CalendarStore> Resource for PrincipalCalendarsResource<C> {
    type UriComponents = ();
    type MemberType = CalendarResource<C>;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        let calendars = self
            .cal_store
            .read()
            .await
            .get_calendars(&self.principal)
            .await?;
        let mut out = Vec::new();
        for calendar in calendars {
            let path = format!("{}/{}", &self.path, &calendar.id);
            out.push(CalendarResource {
                cal_store: self.cal_store.clone(),
                calendar,
                path,
                prefix: self.prefix.clone(),
                principal: self.principal.clone(),
            })
        }
        Ok(out)
    }

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        _uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();
        Ok(Self {
            cal_store,
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
            "resourcetype" => write_resourcetype(writer, vec!["principal", "collection"])?,
            "current-user-principal" | "principal-URL" => {
                write_href_prop(
                    writer,
                    prop,
                    &format!("{}/{}/", self.prefix, self.principal),
                )?;
            }
            "calendar-home-set" | "calendar-user-address-set" => {
                writer
                    .create_element(&format!("C:{prop}"))
                    .write_inner_content(|writer| {
                        writer
                            .create_element("href")
                            .write_text_content(BytesText::new(&format!(
                                "{}/{}/",
                                self.prefix, self.principal
                            )))?;
                        Ok(())
                    })?;
            }
            "allprops" => {}
            _ => {
                return Err(anyhow!("invalid prop"));
            }
        };
        Ok(())
    }

    fn list_dead_props() -> Vec<&'static str> {
        vec![
            "resourcetype",
            "current-user-principal",
            "principal-URL",
            "calendar-home-set",
            "calendar-user-address-set",
        ]
    }
}
