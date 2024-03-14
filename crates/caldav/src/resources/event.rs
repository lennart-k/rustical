use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::{resource::Resource, xml_snippets::TextNode};
use rustical_store::calendar::CalendarStore;
use rustical_store::event::Event;
use serde::Serialize;
use std::sync::Arc;
use strum::{EnumProperty, EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

pub struct EventResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub event: Event,
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, EnumProperty, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum EventProp {
    Getetag,
    #[strum(props(tagname = "C:calendar-data"))]
    CalendarData,
    Getcontenttype,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PrincipalPropResponse {
    Getetag(TextNode),
    CalendarData(TextNode),
    Getcontenttype(TextNode),
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> Resource for EventResource<C> {
    type UriComponents = (String, String, String); // principal, calendar, event
    type MemberType = Self;
    type PropType = EventProp;
    type PropResponse = PrincipalPropResponse;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn acquire_from_request(
        req: HttpRequest,
        _auth_info: AuthInfo,
        uri_components: Self::UriComponents,
        _prefix: String,
    ) -> Result<Self> {
        let (_principal, cid, uid) = uri_components;

        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        let event = cal_store.read().await.get_event(&cid, &uid).await?;

        Ok(Self {
            cal_store,
            event,
            path: req.path().to_string(),
        })
    }

    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            EventProp::Getetag => Ok(PrincipalPropResponse::Getetag(TextNode(Some(
                self.event.get_etag(),
            )))),
            EventProp::CalendarData => Ok(PrincipalPropResponse::CalendarData(TextNode(Some(
                self.event.get_ics(),
            )))),
            EventProp::Getcontenttype => Ok(PrincipalPropResponse::Getcontenttype(TextNode(Some(
                "text/calendar;charset=utf-8".to_owned(),
            )))),
        }
    }
}
