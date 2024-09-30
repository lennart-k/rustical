use crate::calendar_object::resource::CalendarObjectResource;
use crate::Error;
use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::model::Calendar;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};
use tokio::sync::RwLock;

use super::prop::{
    Resourcetype, SupportedCalendarComponent, SupportedCalendarComponentSet, SupportedCalendarData,
    SupportedReportSet, UserPrivilegeSet,
};

pub struct CalendarResourceService<C: CalendarStore + ?Sized> {
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
    SyncToken,
    Getctag,
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
    CalendarOrder(Option<i64>),
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
    MaxResourceSize(i64),
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    SupportedReportSet(SupportedReportSet),
    SyncToken(String),
    Getctag(String),
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for CalendarProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, Debug, From, Into)]
pub struct CalendarResource(Calendar);

impl Resource for CalendarResource {
    type PropName = CalendarPropName;
    type Prop = CalendarProp;
    type Error = Error;

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropName::Resourcetype => CalendarProp::Resourcetype(Resourcetype::default()),
            CalendarPropName::CurrentUserPrincipal => CalendarProp::CurrentUserPrincipal(
                HrefElement::new(format!("{}/user/{}/", prefix, self.0.principal)),
            ),
            CalendarPropName::Owner => CalendarProp::Owner(HrefElement::new(format!(
                "{}/user/{}/",
                prefix, self.0.principal
            ))),
            CalendarPropName::Displayname => CalendarProp::Displayname(self.0.displayname.clone()),
            CalendarPropName::CalendarColor => CalendarProp::CalendarColor(self.0.color.clone()),
            CalendarPropName::CalendarDescription => {
                CalendarProp::CalendarDescription(self.0.description.clone())
            }
            CalendarPropName::CalendarTimezone => {
                CalendarProp::CalendarTimezone(self.0.timezone.clone())
            }
            CalendarPropName::CalendarOrder => CalendarProp::CalendarOrder(Some(self.0.order)),
            CalendarPropName::SupportedCalendarComponentSet => {
                CalendarProp::SupportedCalendarComponentSet(SupportedCalendarComponentSet {
                    comp: vec![
                        SupportedCalendarComponent {
                            name: "VEVENT".to_owned(),
                        },
                        SupportedCalendarComponent {
                            name: "VTODO".to_owned(),
                        },
                    ],
                })
            }
            CalendarPropName::SupportedCalendarData => {
                CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
            }
            CalendarPropName::Getcontenttype => {
                CalendarProp::Getcontenttype("text/calendar;charset=utf-8".to_owned())
            }
            CalendarPropName::MaxResourceSize => CalendarProp::MaxResourceSize(10000000),
            CalendarPropName::CurrentUserPrivilegeSet => {
                CalendarProp::CurrentUserPrivilegeSet(UserPrivilegeSet::default())
            }
            CalendarPropName::SupportedReportSet => {
                CalendarProp::SupportedReportSet(SupportedReportSet::default())
            }
            CalendarPropName::SyncToken => CalendarProp::SyncToken(self.0.format_synctoken()),
            CalendarPropName::Getctag => CalendarProp::Getctag(self.0.format_synctoken()),
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarProp::Resourcetype(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::CurrentUserPrincipal(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Owner(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Displayname(displayname) => {
                self.0.displayname = displayname;
                Ok(())
            }
            CalendarProp::CalendarColor(color) => {
                self.0.color = color;
                Ok(())
            }
            CalendarProp::CalendarDescription(description) => {
                self.0.description = description;
                Ok(())
            }
            CalendarProp::CalendarTimezone(timezone) => {
                self.0.timezone = timezone;
                Ok(())
            }
            CalendarProp::CalendarOrder(order) => {
                self.0.order = order.unwrap_or_default();
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
            CalendarProp::SyncToken(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Getctag(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Invalid => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: Self::PropName) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarPropName::Resourcetype => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::CurrentUserPrincipal => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Owner => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Displayname => {
                self.0.displayname = None;
                Ok(())
            }
            CalendarPropName::CalendarColor => {
                self.0.color = None;
                Ok(())
            }
            CalendarPropName::CalendarDescription => {
                self.0.description = None;
                Ok(())
            }
            CalendarPropName::CalendarTimezone => {
                self.0.timezone = None;
                Ok(())
            }
            CalendarPropName::CalendarOrder => {
                self.0.order = 0;
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
            CalendarPropName::SyncToken => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Getctag => Err(rustical_dav::Error::PropReadOnly),
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarResourceService<C> {
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Error> {
        if self.principal != principal {
            return Err(Error::Unauthorized);
        }
        let calendar = self
            .cal_store
            .read()
            .await
            .get_calendar(&self.principal, &self.calendar_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(calendar.into())
    }

    async fn get_members(&self) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        // As of now the calendar resource has no members since events are shown with REPORT
        Ok(self
            .cal_store
            .read()
            .await
            .get_objects(&self.principal, &self.calendar_id)
            .await?
            .into_iter()
            .map(|event| (format!("{}/{}", self.path, &event.get_uid()), event.into()))
            .collect())
    }

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            path: req.path().to_owned(),
            principal: path_components.0,
            calendar_id: path_components.1,
            cal_store,
        })
    }

    async fn save_resource(&self, file: Self::Resource) -> Result<(), Self::Error> {
        self.cal_store
            .write()
            .await
            .update_calendar(
                self.principal.to_owned(),
                self.calendar_id.to_owned(),
                file.into(),
            )
            .await?;
        Ok(())
    }

    async fn delete_resource(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.cal_store
            .write()
            .await
            .delete_calendar(&self.principal, &self.calendar_id, use_trashbin)
            .await?;
        Ok(())
    }
}
