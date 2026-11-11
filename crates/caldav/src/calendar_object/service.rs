use crate::calendar::resource::objects_read_only;
use crate::{
    CalDavConfig, CalDavPrincipalUri, Error,
    calendar_object::{
        methods::{get_event, put_event},
        resource::CalendarObjectResource,
    },
};
use async_trait::async_trait;
use axum::{extract::Request, handler::Handler, response::Response};
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::access::{self, Resolved};
use rustical_store::shared::{IfMatch, IfNoneMatch};
use rustical_store::{Calendar, CalendarStore, Preconditions, ShareStore, auth::Principal, shared};
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
    pub(crate) share_store: Arc<dyn ShareStore>,
    pub(crate) config: Arc<CalDavConfig>,
}

impl<C: CalendarStore> Clone for CalendarObjectResourceService<C> {
    fn clone(&self) -> Self {
        Self {
            cal_store: self.cal_store.clone(),
            share_store: self.share_store.clone(),
            config: self.config.clone(),
        }
    }
}

impl<C: CalendarStore> CalendarObjectResourceService<C> {
    pub fn new(
        cal_store: Arc<C>,
        share_store: Arc<dyn ShareStore>,
        config: Arc<CalDavConfig>,
    ) -> Self {
        Self {
            cal_store,
            share_store,
            config,
        }
    }

    /// The calendar of an object path
    pub(crate) async fn calendar(
        &self,
        principal: &str,
        calendar_id: &str,
        show_deleted: bool,
    ) -> Result<(Resolved, Calendar), Error> {
        Ok(access::resolve_calendar(
            self.share_store.as_ref(),
            self.cal_store.as_ref(),
            principal,
            calendar_id,
            show_deleted,
        )
        .await?)
    }
}

#[async_trait]
impl<C: CalendarStore> ResourceService for CalendarObjectResourceService<C> {
    type PathComponents = CalendarObjectPathComponents;
    type Resource = CalendarObjectResource;
    type MemberType = CalendarObjectResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access";

    async fn get_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            calendar_id,
            object_id,
        }: &Self::PathComponents,
        show_deleted: bool,
    ) -> Result<Self::Resource, Self::Error> {
        let (resolved, calendar) = self.calendar(principal, calendar_id, true).await?;
        let object =
            shared::object(self.cal_store.as_ref(), &resolved, object_id, show_deleted).await?;
        let level = shared::level(&resolved, &object);
        Ok(CalendarObjectResource::new(
            object,
            object_id.to_owned(),
            principal,
            &resolved,
            objects_read_only(self.cal_store.as_ref(), &calendar),
            level,
        ))
    }

    async fn delete_resource(
        &self,
        path: &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.delete_resource_with_preconditions(path, use_trashbin, None, None)
            .await
    }

    async fn delete_resource_with_preconditions(
        &self,
        CalendarObjectPathComponents {
            principal,
            calendar_id,
            object_id,
        }: &Self::PathComponents,
        use_trashbin: bool,
        if_match: Option<&IfMatch>,
        if_none_match: Option<&IfNoneMatch>,
    ) -> Result<(), Self::Error> {
        let (resolved, _calendar) = self.calendar(principal, calendar_id, true).await?;
        // Through a share checked again on the sharee's view, in the transaction of the deletion
        shared::delete_object(
            self.cal_store.as_ref(),
            self.share_store.as_ref(),
            &resolved,
            object_id,
            use_trashbin,
            Preconditions {
                if_match,
                if_none_match,
            },
        )
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
    name.strip_suffix(".ics").map_or_else(
        || Err(serde::de::Error::custom("Missing .ics extension")),
        |object_id| Ok(object_id.to_owned()),
    )
}
