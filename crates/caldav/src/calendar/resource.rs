use crate::event::resource::EventFile;
use crate::Error;
use actix_web::{web::Data, HttpRequest};
use anyhow::anyhow;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::calendar::Calendar;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};
use tokio::sync::RwLock;

use super::prop::{
    Resourcetype, SupportedCalendarComponent, SupportedCalendarComponentSet, SupportedCalendarData,
    SupportedReportSet, UserPrivilegeSet,
};

pub struct CalendarResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub principal: String,
    pub calendar_id: String,
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CalendarPropName {
    Resourcetype,
    CurrentUserPrincipal,
    Owner,
    Displayname,
    CalendarColor,
    CalendarDescription,
    CalendarTimezone,
    CalendarOrder,
    SupportedCalendarComponentSet,
    SupportedCalendarData,
    Getcontenttype,
    CurrentUserPrivilegeSet,
    MaxResourceSize,
    SupportedReportSet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarProp {
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
    Owner(HrefElement),
    Displayname(Option<String>),
    #[serde(rename = "IC:calendar-color", alias = "calendar-color")]
    CalendarColor(Option<String>),
    #[serde(rename = "C:calendar-description", alias = "calendar-description")]
    CalendarDescription(Option<String>),
    #[serde(rename = "C:calendar-timezone", alias = "calendar-timezone")]
    CalendarTimezone(Option<String>),
    #[serde(rename = "IC:calendar-order", alias = "calendar-order")]
    CalendarOrder(Option<String>),
    #[serde(
        rename = "C:supported-calendar-component-set",
        alias = "supported-calendar-component-set"
    )]
    SupportedCalendarComponentSet(SupportedCalendarComponentSet),
    #[serde(
        rename = "C:supported-calendar-data",
        alias = "supported-calendar-data"
    )]
    SupportedCalendarData(SupportedCalendarData),
    Getcontenttype(String),
    MaxResourceSize(String),
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    SupportedReportSet(SupportedReportSet),
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for CalendarProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, Debug)]
pub struct CalendarFile {
    pub calendar: Calendar,
    pub principal: String,
}

impl Resource for CalendarFile {
    type PropName = CalendarPropName;
    type Prop = CalendarProp;
    type Error = Error;

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropName::Resourcetype => CalendarProp::Resourcetype(Resourcetype::default()),
            CalendarPropName::CurrentUserPrincipal => CalendarProp::CurrentUserPrincipal(
                HrefElement::new(format!("{}/user/{}/", prefix, self.principal)),
            ),
            CalendarPropName::Owner => CalendarProp::Owner(HrefElement::new(format!(
                "{}/user/{}/",
                prefix, self.principal
            ))),
            CalendarPropName::Displayname => {
                CalendarProp::Displayname(self.calendar.displayname.clone())
            }
            CalendarPropName::CalendarColor => {
                CalendarProp::CalendarColor(self.calendar.color.clone())
            }
            CalendarPropName::CalendarDescription => {
                CalendarProp::CalendarDescription(self.calendar.description.clone())
            }
            CalendarPropName::CalendarTimezone => {
                CalendarProp::CalendarTimezone(self.calendar.timezone.clone())
            }
            CalendarPropName::CalendarOrder => {
                CalendarProp::CalendarOrder(format!("{}", self.calendar.order).into())
            }
            CalendarPropName::SupportedCalendarComponentSet => {
                CalendarProp::SupportedCalendarComponentSet(SupportedCalendarComponentSet {
                    comp: vec![SupportedCalendarComponent {
                        name: "VEVENT".to_owned(),
                    }],
                })
            }
            CalendarPropName::SupportedCalendarData => {
                CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
            }
            CalendarPropName::Getcontenttype => {
                CalendarProp::Getcontenttype("text/calendar;charset=utf-8".to_owned())
            }
            CalendarPropName::MaxResourceSize => {
                CalendarProp::MaxResourceSize("10000000".to_owned())
            }
            CalendarPropName::CurrentUserPrivilegeSet => {
                CalendarProp::CurrentUserPrivilegeSet(UserPrivilegeSet::default())
            }
            CalendarPropName::SupportedReportSet => {
                CalendarProp::SupportedReportSet(SupportedReportSet::default())
            }
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarProp::Resourcetype(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::CurrentUserPrincipal(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Owner(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Displayname(displayname) => {
                self.calendar.displayname = displayname;
                Ok(())
            }
            CalendarProp::CalendarColor(color) => {
                self.calendar.color = color;
                Ok(())
            }
            CalendarProp::CalendarDescription(description) => {
                self.calendar.description = description;
                Ok(())
            }
            CalendarProp::CalendarTimezone(timezone) => {
                self.calendar.timezone = timezone;
                Ok(())
            }
            CalendarProp::CalendarOrder(order) => {
                self.calendar.order = match order {
                    Some(order) => order.parse().map_err(|_e| anyhow!("invalid order"))?,
                    None => 0,
                };
                Ok(())
            }
            CalendarProp::SupportedCalendarComponentSet(_) => {
                Err(rustical_dav::Error::PropReadOnly)
            }
            CalendarProp::SupportedCalendarData(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Getcontenttype(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::MaxResourceSize(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::CurrentUserPrivilegeSet(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Invalid => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: Self::PropName) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarPropName::Resourcetype => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::CurrentUserPrincipal => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Owner => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Displayname => {
                self.calendar.displayname = None;
                Ok(())
            }
            CalendarPropName::CalendarColor => {
                self.calendar.color = None;
                Ok(())
            }
            CalendarPropName::CalendarDescription => {
                self.calendar.description = None;
                Ok(())
            }
            CalendarPropName::CalendarTimezone => {
                self.calendar.timezone = None;
                Ok(())
            }
            CalendarPropName::CalendarOrder => {
                self.calendar.order = 0;
                Ok(())
            }
            CalendarPropName::SupportedCalendarComponentSet => {
                Err(rustical_dav::Error::PropReadOnly)
            }
            CalendarPropName::SupportedCalendarData => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Getcontenttype => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::MaxResourceSize => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::CurrentUserPrivilegeSet => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::SupportedReportSet => Err(rustical_dav::Error::PropReadOnly),
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarResource<C> {
    type MemberType = EventFile;
    type PathComponents = (String, String); // principal, calendar_id
    type File = CalendarFile;
    type Error = Error;

    async fn get_file(&self) -> Result<Self::File, Error> {
        let calendar = self
            .cal_store
            .read()
            .await
            .get_calendar(&self.principal, &self.calendar_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(CalendarFile {
            calendar,
            principal: self.principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        _auth_info: AuthInfo,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        // As of now the calendar resource has no members since events are shown with REPORT
        Ok(self
            .cal_store
            .read()
            .await
            .get_events(&self.principal, &self.calendar_id)
            .await?
            .into_iter()
            .map(|event| {
                (
                    format!("{}/{}", self.path, &event.get_uid()),
                    EventFile { event },
                )
            })
            .collect())
    }

    async fn new(
        req: &HttpRequest,
        auth_info: &AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        Ok(Self {
            path: req.path().to_owned(),
            principal: auth_info.user_id.to_owned(),
            calendar_id: path_components.1,
            cal_store,
        })
    }

    async fn save_file(&self, file: Self::File) -> Result<(), Self::Error> {
        self.cal_store
            .write()
            .await
            .update_calendar(
                self.principal.to_owned(),
                self.calendar_id.to_owned(),
                file.calendar,
            )
            .await?;
        Ok(())
    }

    async fn delete_file(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.cal_store
            .write()
            .await
            .delete_calendar(&self.principal, &self.calendar_id, use_trashbin)
            .await?;
        Ok(())
    }
}
