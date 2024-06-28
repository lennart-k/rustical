use crate::Error;
use actix_web::{web::Data, HttpRequest};
use anyhow::anyhow;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_store::event::Event;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};
use tokio::sync::RwLock;

pub struct EventResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub principal: String,
    pub cid: String,
    pub uid: String,
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum EventPropName {
    Getetag,
    CalendarData,
    Getcontenttype,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum EventProp {
    Getetag(String),
    #[serde(rename = "C:calendar-data")]
    CalendarData(String),
    Getcontenttype(String),
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for EventProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone)]
pub struct EventFile {
    pub event: Event,
}

impl Resource for EventFile {
    type PropName = EventPropName;
    type Prop = EventProp;
    type Error = Error;

    fn get_prop(&self, _prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        match prop {
            EventPropName::Getetag => Ok(EventProp::Getetag(self.event.get_etag())),
            EventPropName::CalendarData => {
                Ok(EventProp::CalendarData(self.event.get_ics().to_owned()))
            }
            EventPropName::Getcontenttype => Ok(EventProp::Getcontenttype(
                "text/calendar;charset=utf-8".to_owned(),
            )),
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for EventResource<C> {
    type PathComponents = (String, String, String); // principal, calendar, event
    type File = EventFile;
    type MemberType = EventFile;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        _auth_info: &AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let (principal, cid, uid) = path_components;

        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            principal,
            cid,
            uid,
            path: req.path().to_string(),
        })
    }

    async fn get_file(&self) -> Result<Self::File, Self::Error> {
        let event = self
            .cal_store
            .read()
            .await
            .get_event(&self.principal, &self.cid, &self.uid)
            .await?;
        Ok(EventFile { event })
    }

    async fn save_file(&self, _file: Self::File) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
