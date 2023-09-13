use std::{io::Write, sync::Arc};

use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::{events::BytesText, Writer};
use rustical_auth::AuthInfo;
use rustical_store::calendar::{Calendar, CalendarStore};
use tokio::sync::RwLock;

use crate::{
    propfind::write_resourcetype,
    proptypes::{write_href_prop, write_string_prop},
    resource::Resource,
};

pub struct CalendarResource<C: CalendarStore> {
    pub cal_store: Arc<RwLock<C>>,
    pub calendar: Calendar,
    pub path: String,
    pub prefix: String,
    pub principal: String,
}

#[async_trait(?Send)]
impl<C: CalendarStore> Resource for CalendarResource<C> {
    type MemberType = Self;
    type UriComponents = (String, String); // principal, calendar_id

    async fn acquire_from_request(
        req: HttpRequest,
        _auth_info: AuthInfo,
        uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        let (principal, cid) = uri_components;
        let calendar = cal_store.read().await.get_calendar(&cid).await?;
        Ok(Self {
            cal_store,
            calendar,
            path: req.path().to_string(),
            prefix,
            principal,
        })
    }

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        // As of now the calendar resource has no members
        Ok(vec![])
    }

    #[inline]
    fn list_dead_props() -> Vec<&'static str> {
        vec![
            "resourcetype",
            "current-user-principal",
            "displayname",
            "supported-calendar-component-set",
            "supported-calendar-data",
            "getcontenttype",
            "calendar-description",
            "owner",
            "calendar-color",
            "current-user-privilege-set",
            "max-resource-size",
        ]
    }
    fn write_prop<W: Write>(&self, writer: &mut Writer<W>, prop: &str) -> Result<()> {
        match prop {
            "resourcetype" => write_resourcetype(writer, vec!["C:calendar", "collection"])?,
            "current-user-principal" | "owner" => {
                write_href_prop(
                    writer,
                    prop,
                    &format!("{}/{}/", self.prefix, self.principal),
                )?;
            }
            "displayname" => {
                let el = writer.create_element("displayname");
                if let Some(name) = self.calendar.clone().name {
                    el.write_text_content(BytesText::new(&name))?;
                } else {
                    el.write_empty()?;
                }
            }
            "calendar-color" => {
                let el = writer.create_element("IC:calendar-color");
                if let Some(color) = self.calendar.clone().color {
                    el.write_text_content(BytesText::new(&color))?;
                } else {
                    el.write_empty()?;
                }
            }
            "calendar-description" => {
                let el = writer.create_element("C:calendar-description");
                if let Some(description) = self.calendar.clone().description {
                    el.write_text_content(BytesText::new(&description))?;
                } else {
                    el.write_empty()?;
                }
            }
            "supported-calendar-component-set" => {
                writer
                    .create_element("C:supported-calendar-component-set")
                    .write_inner_content(|writer| {
                        writer
                            .create_element("C:comp")
                            .with_attribute(("name", "VEVENT"))
                            .write_empty()?;
                        Ok(())
                    })?;
            }
            "supported-calendar-data" => {
                writer
                    .create_element("C:supported-calendar-data")
                    .write_inner_content(|writer| {
                        // <cal:calendar-data content-type="text/calendar" version="2.0" />
                        writer
                            .create_element("C:calendar-data")
                            .with_attributes(vec![
                                ("content-type", "text/calendar"),
                                ("version", "2.0"),
                            ])
                            .write_empty()?;
                        Ok(())
                    })?;
            }
            "getcontenttype" => {
                write_string_prop(writer, "getcontenttype", "text/calendar")?;
            }
            "max-resource-size" => {
                write_string_prop(writer, "max-resource-size", "10000000")?;
            }
            "current-user-privilege-set" => {
                writer
                    .create_element("current-user-privilege-set")
                    // These are just hard-coded for now and will possibly change in the future
                    .write_inner_content(|writer| {
                        for privilege in [
                            "read",
                            "read-acl",
                            "write",
                            "write-acl",
                            "write-content",
                            "read-current-user-privilege-set",
                            "bind",
                            "unbind",
                        ] {
                            writer
                                .create_element("privilege")
                                .write_inner_content(|writer| {
                                    writer.create_element(privilege).write_empty()?;
                                    Ok(())
                                })?;
                        }
                        Ok(())
                    })?;
            }
            _ => {
                return Err(anyhow!("invalid prop"));
            }
        };
        Ok(())
    }
}
