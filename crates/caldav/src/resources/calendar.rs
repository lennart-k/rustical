use std::{io::Write, sync::Arc};

use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use quick_xml::{events::BytesText, Writer};
use rustical_auth::AuthInfo;
use rustical_store::calendar::{Calendar, CalendarStore};
use strum::{EnumProperty, EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

use crate::{
    proptypes::{write_href_prop, write_string_prop},
    tagname::TagName,
};
use rustical_dav::{resource::Resource, xml_snippets::write_resourcetype};

pub struct CalendarResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub calendar: Calendar,
    pub path: String,
    pub prefix: String,
    pub principal: String,
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, EnumProperty)]
#[strum(serialize_all = "kebab-case")]
pub enum CalendarProp {
    Resourcetype,
    CurrentUserPrincipal,
    Owner,
    Displayname,
    #[strum(props(tagname = "IC:calendar-color"))]
    CalendarColor,
    #[strum(props(tagname = "C:calendar-description"))]
    CalendarDescription,
    #[strum(props(tagname = "C:supported-calendar-component-set"))]
    SupportedCalendarComponentSet,
    #[strum(props(tagname = "C:supported-calendar-data"))]
    SupportedCalendarData,
    Getcontenttype,
    CurrentUserPrivilegeSet,
    MaxResourceSize,
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> Resource for CalendarResource<C> {
    type MemberType = Self;
    type UriComponents = (String, String); // principal, calendar_id
    type PropType = CalendarProp;

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

    fn write_prop<W: Write>(&self, writer: &mut Writer<W>, prop: Self::PropType) -> Result<()> {
        match prop {
            CalendarProp::Resourcetype => {
                write_resourcetype(writer, vec!["C:calendar", "collection"])?
            }
            CalendarProp::CurrentUserPrincipal | CalendarProp::Owner => {
                write_href_prop(
                    writer,
                    prop.tagname(),
                    &format!("{}/{}/", self.prefix, self.principal),
                )?;
            }
            CalendarProp::Displayname => {
                let el = writer.create_element(prop.tagname());
                if let Some(name) = self.calendar.clone().name {
                    el.write_text_content(BytesText::new(&name))?;
                } else {
                    el.write_empty()?;
                }
            }
            CalendarProp::CalendarColor => {
                let el = writer.create_element(prop.tagname());
                if let Some(color) = self.calendar.clone().color {
                    el.write_text_content(BytesText::new(&color))?;
                } else {
                    el.write_empty()?;
                }
            }
            CalendarProp::CalendarDescription => {
                let el = writer.create_element(prop.tagname());
                if let Some(description) = self.calendar.clone().description {
                    el.write_text_content(BytesText::new(&description))?;
                } else {
                    el.write_empty()?;
                }
            }
            CalendarProp::SupportedCalendarComponentSet => {
                writer
                    .create_element(prop.tagname())
                    .write_inner_content(|writer| {
                        writer
                            .create_element("C:comp")
                            .with_attribute(("name", "VEVENT"))
                            .write_empty()?;
                        Ok::<(), quick_xml::Error>(())
                    })?;
            }
            CalendarProp::SupportedCalendarData => {
                writer
                    .create_element(prop.tagname())
                    .write_inner_content(|writer| {
                        // <cal:calendar-data content-type="text/calendar" version="2.0" />
                        writer
                            .create_element("C:calendar-data")
                            .with_attributes(vec![
                                ("content-type", "text/calendar"),
                                ("version", "2.0"),
                            ])
                            .write_empty()?;
                        Ok::<(), quick_xml::Error>(())
                    })?;
            }
            CalendarProp::Getcontenttype => {
                write_string_prop(writer, prop.tagname(), "text/calendar")?;
            }
            CalendarProp::MaxResourceSize => {
                write_string_prop(writer, prop.tagname(), "10000000")?;
            }
            CalendarProp::CurrentUserPrivilegeSet => {
                writer
                    .create_element(prop.tagname())
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
                                    Ok::<(), quick_xml::Error>(())
                                })?;
                        }
                        Ok::<(), quick_xml::Error>(())
                    })?;
            }
        };
        Ok(())
    }
}
