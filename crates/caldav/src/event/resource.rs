use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::error::Error;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml_snippets::TextNode;
use rustical_store::event::Event;
use rustical_store::CalendarStore;
use serde::Serialize;
use std::sync::Arc;
use strum::{EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

pub struct EventResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub cid: String,
    pub uid: String,
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum EventProp {
    Getetag,
    CalendarData,
    Getcontenttype,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalPropResponse {
    Getetag(TextNode),
    CalendarData(TextNode),
    Getcontenttype(TextNode),
}

pub struct EventFile {
    pub event: Event,
    pub path: String,
}

impl Resource for EventFile {
    type PropType = EventProp;
    type PropResponse = PrincipalPropResponse;

    fn get_path(&self) -> &str {
        &self.path
    }

    fn get_prop(&self, _prefix: &str, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            EventProp::Getetag => Ok(PrincipalPropResponse::Getetag(TextNode(Some(
                self.event.get_etag(),
            )))),
            EventProp::CalendarData => Ok(PrincipalPropResponse::CalendarData(TextNode(Some(
                self.event.get_ics().to_owned(),
            )))),
            EventProp::Getcontenttype => Ok(PrincipalPropResponse::Getcontenttype(TextNode(Some(
                "text/calendar;charset=utf-8".to_owned(),
            )))),
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for EventResource<C> {
    type PathComponents = (String, String, String); // principal, calendar, event
    type File = EventFile;
    type MemberType = EventFile;

    async fn get_members(&self, _auth_info: AuthInfo) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn new(
        req: HttpRequest,
        _auth_info: AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, Error> {
        let (_principal, cid, uid) = path_components;

        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            cid,
            uid,
            path: req.path().to_string(),
        })
    }

    async fn get_file(&self) -> Result<Self::File> {
        let event = self
            .cal_store
            .read()
            .await
            .get_event(&self.cid, &self.uid)
            .await?;
        Ok(EventFile {
            event,
            path: self.path.to_owned(),
        })
    }
}
