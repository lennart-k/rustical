use crate::{
    CalDavPrincipalUri, Error,
    calendar_object::{
        methods::{get_event, put_event},
        resource::CalendarObjectResource,
    },
};
use async_trait::async_trait;
use axum::{extract::Request, handler::Handler, response::Response};
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::{CalendarStore, auth::User};
use serde::{Deserialize, Deserializer};
use std::{convert::Infallible, sync::Arc};
use tower::Service;

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarObjectPathComponents {
    pub principal: String,
    pub calendar_id: String,
    #[serde(deserialize_with = "deserialize_ics_name")]
    pub object_id: String,
}

pub struct CalendarObjectResourceService<C: CalendarStore> {
    pub(crate) cal_store: Arc<C>,
}

impl<C: CalendarStore> Clone for CalendarObjectResourceService<C> {
    fn clone(&self) -> Self {
        Self {
            cal_store: self.cal_store.clone(),
        }
    }
}

impl<C: CalendarStore> CalendarObjectResourceService<C> {
    pub fn new(cal_store: Arc<C>) -> Self {
        Self { cal_store }
    }
}

#[async_trait]
impl<C: CalendarStore> ResourceService for CalendarObjectResourceService<C> {
    type PathComponents = CalendarObjectPathComponents;
    type Resource = CalendarObjectResource;
    type MemberType = CalendarObjectResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access";
    const IS_COLLECTION: bool = false;

    async fn get_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            calendar_id,
            object_id,
        }: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let object = self
            .cal_store
            .get_object(principal, calendar_id, object_id)
            .await?;
        Ok(CalendarObjectResource {
            object,
            principal: principal.to_owned(),
        })
    }

    async fn delete_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            calendar_id,
            object_id,
        }: &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .delete_object(principal, calendar_id, object_id, use_trashbin)
            .await?;
        Ok(())
    }
}

impl<C: CalendarStore> AxumMethods for CalendarObjectResourceService<C> {
    fn get() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(get_event::<C>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
    fn put() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(put_event::<C>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
}

fn deserialize_ics_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name: String = Deserialize::deserialize(deserializer)?;
    if let Some(object_id) = name.strip_suffix(".ics") {
        Ok(object_id.to_owned())
    } else {
        Err(serde::de::Error::custom("Missing .ics extension"))
    }
}
